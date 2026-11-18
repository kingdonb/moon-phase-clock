package com.urmanac.moonphase

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class WasmEngineTest {

    // Published 2026 instants (UTC), as UNIX timestamps
    private val annularEclipseNewMoon = 1771329660.0 // 2026-02-17 12:01
    private val totalLunarEclipseFullMoon = 1772537880.0 // 2026-03-03 11:38
    private val betweenThem = 1771933770.0 // midway: about First Quarter
    private val crescentDay = 1771675200.0 // 2026-02-21 12:00, four days after new

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
        // Full Moon: 2025-01-13 22:27:00 UTC
        val resultJson = newEngine().calculatePhase(1736807220.0)

        println("WASM Result: $resultJson")

        assertTrue("Expected Full Moon in JSON", resultJson.contains("\"phase_name\": \"Full Moon\""))
        assertTrue("Expected Torment Multiplier 3.0", resultJson.contains("\"torment_multiplier\": 3.0"))
        assertTrue("Expected a nearly full disc", numberField(resultJson, "illumination") > 0.999)
    }

    @Test
    fun `eclipses are new and full to four decimals`() {
        val engine = newEngine()

        val newMoon = engine.calculatePhase(annularEclipseNewMoon)
        assertEquals("New Moon", stringField(newMoon, "phase_name"))
        assertEquals(0.0, numberField(newMoon, "illumination"), 0.0001)

        val fullMoon = engine.calculatePhase(totalLunarEclipseFullMoon)
        assertEquals("Full Moon", stringField(fullMoon, "phase_name"))
        assertEquals(1.0, numberField(fullMoon, "illumination"), 0.0001)
    }

    @Test
    fun `midway between new and full is about first quarter`() {
        val json = newEngine().calculatePhase(betweenThem)
        assertEquals("First Quarter", stringField(json, "phase_name"))
        assertEquals(0.5, numberField(json, "illumination"), 0.05)
        assertTrue("four decimals: $json", Regex("\"illumination\": \\d\\.\\d{4}}").containsMatchIn(json))
    }

    @Test
    fun `repeated calls reuse the result buffer cleanly`() {
        val engine = newEngine()
        // Longest result first, then the shortest: no stale bytes may leak through.
        assertEquals("Waxing Crescent", stringField(engine.calculatePhase(crescentDay), "phase_name"))
        val data = engine.calculatePhase(annularEclipseNewMoon)
        assertTrue("clean JSON: $data", data.endsWith("}"))
        assertEquals("New Moon", stringField(data, "phase_name"))
        assertEquals(1.0, numberField(data, "torment_multiplier"), 0.0)
    }
}
