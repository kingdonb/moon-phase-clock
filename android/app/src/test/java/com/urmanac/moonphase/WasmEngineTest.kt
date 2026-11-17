package com.urmanac.moonphase

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class WasmEngineTest {

    // Mirrors the constants in brain/src/lib.rs, only to pick test instants.
    private val knownNewMoon = 947182440.0
    private val synodicMonthSeconds = 29.53058770576 * 24 * 60 * 60

    /** A timestamp at the given fraction of a lunation (0.0 = New, 0.5 = Full). */
    private fun atCyclePosition(position: Double) =
        knownNewMoon + (300 + position) * synodicMonthSeconds

    // android.jar's org.json is a stub under local unit tests, so pull the
    // fields out of the brain's flat JSON directly.
    private fun stringField(json: String, name: String) =
        Regex("\"$name\": \"([^\"]*)\"").find(json)?.groupValues?.get(1)
            ?: throw AssertionError("No string field $name in $json")

    private fun numberField(json: String, name: String) =
        Regex("\"$name\": ([0-9.]+)").find(json)?.groupValues?.get(1)?.toDouble()
            ?: throw AssertionError("No number field $name in $json")

    private fun newEngine(): WasmEngine {
        // Resolved from src/main/assets, the same binary that ships in the app.
        val wasmStream = this::class.java.classLoader?.getResourceAsStream("moon-phase.wasm")
            ?: throw IllegalStateException("Could not find moon-phase.wasm on the test classpath")
        return WasmEngine(wasmStream)
    }

    @Test
    fun `test wasm engine loads and calculates full moon`() {
        // We simulate passing a known Full Moon timestamp to the engine.
        // Full Moon example: ~2025-01-13 22:27:00 UTC (1736807220.0)
        val resultJson = newEngine().calculatePhase(1736807220.0)

        println("WASM Result: $resultJson")

        // Assert the Torment Multiplier is Maxed out and it identified the Full Moon
        assertTrue("Expected Full Moon in JSON", resultJson.contains("\"phase_name\": \"Full Moon\""))
        assertTrue("Expected Torment Multiplier 3.0", resultJson.contains("\"torment_multiplier\": 3.0"))

        val illumination = numberField(resultJson, "illumination")
        assertTrue("Expected a nearly full disc, got $illumination", illumination > 0.98)
    }

    @Test
    fun `illumination follows the cosine law with four decimals`() {
        val engine = newEngine()
        val expected = mapOf(
            0.125 to ("Waxing Crescent" to 0.1464),
            0.25 to ("First Quarter" to 0.5000),
            0.375 to ("Waxing Gibbous" to 0.8536),
            0.5 to ("Full Moon" to 1.0000),
            0.625 to ("Waning Gibbous" to 0.8536),
            0.75 to ("Last Quarter" to 0.5000),
            0.875 to ("Waning Crescent" to 0.1464),
        )
        for ((position, phaseAndIllumination) in expected) {
            val (phase, illumination) = phaseAndIllumination
            val data = engine.calculatePhase(atCyclePosition(position))
            assertEquals("phase at $position", phase, stringField(data, "phase_name"))
            assertEquals("illumination at $position", illumination, numberField(data, "illumination"), 0.00005)
        }
    }

    @Test
    fun `repeated calls reuse the result buffer cleanly`() {
        val engine = newEngine()
        // Longest result first, then the shortest: no stale bytes may leak through.
        engine.calculatePhase(atCyclePosition(0.1))
        val data = engine.calculatePhase(atCyclePosition(0.0001))
        assertEquals("New Moon", stringField(data, "phase_name"))
        assertEquals(1.0, numberField(data, "torment_multiplier"), 0.0)
        assertEquals(0.0, numberField(data, "illumination"), 0.00005)
    }
}
