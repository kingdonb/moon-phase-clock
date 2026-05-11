package com.useless.moonphase

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.clipPath
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.json.JSONObject

class MainActivity : ComponentActivity() {
    private var currentPhaseName: String? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        val wasmEngine = try {
            WasmEngine(assets.open("moon-phase.wasm"))
        } catch (e: Exception) {
            null
        }
        
        setContent {
            MaterialTheme(colorScheme = darkColorScheme()) {
                Surface(
                    modifier = Modifier.fillMaxSize(),
                    color = Color(0xFF0D0D0D)
                ) {
                    MoonPhaseScreen(wasmEngine) { phase ->
                        currentPhaseName = phase
                    }
                }
            }
        }
    }

    override fun onStop() {
        super.onStop()
        // Update the app icon only when the user leaves the app to prevent the system
        // from killing the process while the app is in the foreground.
        currentPhaseName?.let { phase ->
            LunarIconManager.updateIcon(this, phase)
        }
    }
}

@Composable
fun MoonPhaseScreen(engine: WasmEngine?, onPhaseCalculated: (String) -> Unit) {
    var moonData by remember { mutableStateOf<JSONObject?>(null) }
    var error by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(Unit) {
        if (engine == null) {
            error = "WASM Brain not found in assets!"
            return@LaunchedEffect
        }
        try {
            val now = System.currentTimeMillis() / 1000.0
            val result = engine.calculatePhase(now)
            val data = JSONObject(result)
            moonData = data
            
            // Notify the activity of the current phase
            data.optString("phase_name")?.let { phase ->
                onPhaseCalculated(phase)
            }
        } catch (e: Exception) {
            error = e.message
        }
    }

    Column(
        modifier = Modifier.fillMaxSize().padding(24.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text(
            text = "THE MOON",
            style = MaterialTheme.typography.labelLarge,
            color = Color.Gray,
            letterSpacing = 4.sp
        )
        
        Spacer(modifier = Modifier.height(32.dp))

        if (error != null) {
            android.util.Log.e("MoonPhase", "WASM Error: $error")
            Text(text = error!!, color = Color.Red)
        } else if (moonData == null) {
            CircularProgressIndicator(color = Color.White)
        } else {
            val phase = moonData!!.getString("phase_name")
            val illum = moonData!!.getDouble("illumination").toFloat()
            val multiplier = moonData!!.getDouble("torment_multiplier")
            
            // Draw the Moon
            MoonView(
                illumination = illum,
                phaseName = phase,
                modifier = Modifier.size(160.dp)
            )

            Spacer(modifier = Modifier.height(32.dp))

            Text(
                text = phase.uppercase(),
                style = MaterialTheme.typography.headlineLarge,
                color = Color.White,
                fontWeight = FontWeight.Bold,
                textAlign = TextAlign.Center
            )
            
            val illumPercent = (illum * 100).toInt()
            Text(
                text = "$illumPercent% ILLUMINATED",
                style = MaterialTheme.typography.bodyLarge,
                color = Color.LightGray
            )

            Spacer(modifier = Modifier.height(48.dp))
            
            // Torment Gauge
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .wrapContentHeight() // Fix: Use wrapContentHeight instead of fixed height
                    .background(Color.White.copy(alpha = 0.05f), RoundedCornerShape(8.dp))
                    .padding(vertical = 24.dp, horizontal = 16.dp), // Increase vertical padding
                contentAlignment = Alignment.Center
            ) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text(
                        text = "TORMENT MULTIPLIER",
                        style = MaterialTheme.typography.labelSmall,
                        color = if (multiplier >= 3.0) Color.Red else Color.Gray
                    )
                    Text(
                        text = "${multiplier}x",
                        style = MaterialTheme.typography.displaySmall,
                        color = if (multiplier >= 3.0) Color.Red else Color.White,
                        fontWeight = FontWeight.Black,
                        lineHeight = 40.sp // Fix: Ensure enough line height for the text
                    )
                }
            }
            
            if (multiplier >= 3.0) {
                Spacer(modifier = Modifier.height(16.dp))
                Text(
                    text = "THE BOAR GOD CHARGES",
                    style = MaterialTheme.typography.labelMedium,
                    color = Color.Red,
                    fontWeight = FontWeight.Bold
                )
            }
        }
    }
}

/**
 * A custom view to draw the moon phase using Canvas.
 */
@Composable
fun MoonView(illumination: Float, phaseName: String, modifier: Modifier = Modifier) {
    androidx.compose.foundation.Canvas(modifier = modifier) {
        val radius = size.minDimension / 2f
        val center = androidx.compose.ui.geometry.Offset(size.width / 2f, size.height / 2f)

        // Draw the dark part of the moon (shadow)
        drawCircle(
            color = Color(0xFF1A1A1A),
            radius = radius,
            center = center
        )

        val isWaxing = phaseName.contains("Waxing") || phaseName == "First Quarter"
        val isWaning = phaseName.contains("Waning") || phaseName == "Last Quarter"
        val isFull = phaseName == "Full Moon"

        val path = androidx.compose.ui.graphics.Path()

        if (isFull) {
            path.addOval(androidx.compose.ui.geometry.Rect(center.x - radius, center.y - radius, center.x + radius, center.y + radius))
        } else if (illumination > 0.01f) {
            if (isWaxing) {
                path.addArc(
                    androidx.compose.ui.geometry.Rect(center.x - radius, center.y - radius, center.x + radius, center.y + radius),
                    -90f,
                    180f
                )
                if (illumination < 0.5f) {
                    val innerWidth = radius * (1f - 2f * illumination)
                    path.addArc(
                        androidx.compose.ui.geometry.Rect(center.x - innerWidth, center.y - radius, center.x + innerWidth, center.y + radius),
                        90f,
                        -180f
                    )
                } else {
                    val innerWidth = radius * (2f * illumination - 1f)
                    path.addArc(
                        androidx.compose.ui.geometry.Rect(center.x - innerWidth, center.y - radius, center.x + innerWidth, center.y + radius),
                        90f,
                        180f
                    )
                }
            } else if (isWaning) {
                path.addArc(
                    androidx.compose.ui.geometry.Rect(center.x - radius, center.y - radius, center.x + radius, center.y + radius),
                    90f,
                    180f
                )
                if (illumination < 0.5f) {
                    val innerWidth = radius * (1f - 2f * illumination)
                    path.addArc(
                        androidx.compose.ui.geometry.Rect(center.x - innerWidth, center.y - radius, center.x + innerWidth, center.y + radius),
                        -90f,
                        -180f
                    )
                } else {
                    val innerWidth = radius * (2f * illumination - 1f)
                    path.addArc(
                        androidx.compose.ui.geometry.Rect(center.x - innerWidth, center.y - radius, center.x + innerWidth, center.y + radius),
                        -90f,
                        180f
                    )
                }
            }
        }

        if (!path.isEmpty) {
            drawPath(path = path, color = Color.White)

            // Draw Craters (Clipped to the lit path)
            clipPath(path) {
                val craterColor = Color(0xFFE0E0E0)
                // Tyco-like crater
                drawCircle(
                    color = craterColor,
                    radius = radius * 0.12f,
                    center = androidx.compose.ui.geometry.Offset(center.x + radius * 0.1f, center.y + radius * 0.5f)
                )
                // Copernicus-like crater
                drawCircle(
                    color = craterColor,
                    radius = radius * 0.15f,
                    center = androidx.compose.ui.geometry.Offset(center.x - radius * 0.3f, center.y - radius * 0.1f)
                )
                // Scattered smaller craters
                drawCircle(
                    color = craterColor,
                    radius = radius * 0.08f,
                    center = androidx.compose.ui.geometry.Offset(center.x + radius * 0.4f, center.y - radius * 0.3f)
                )
                drawCircle(
                    color = craterColor,
                    radius = radius * 0.06f,
                    center = androidx.compose.ui.geometry.Offset(center.x - radius * 0.1f, center.y + radius * 0.2f)
                )
                drawCircle(
                    color = craterColor,
                    radius = radius * 0.10f,
                    center = androidx.compose.ui.geometry.Offset(center.x + radius * 0.5f, center.y + radius * 0.1f)
                )
                drawCircle(
                    color = craterColor,
                    radius = radius * 0.07f,
                    center = androidx.compose.ui.geometry.Offset(center.x - radius * 0.5f, center.y - radius * 0.4f)
                )
            }
        }
    }
}
