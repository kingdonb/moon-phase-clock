package com.useless.moonphase

import android.content.ComponentName
import android.content.Context
import android.content.pm.PackageManager

import android.util.Log

/**
 * Handles the swapping of app icons based on the moon phase.
 */
object LunarIconManager {

    private val phaseToAlias = mapOf(
        "Full Moon" to ".MainActivityFull",
        "Waxing Crescent" to ".MainActivityWaxingCrescent",
        "First Quarter" to ".MainActivityFirstQuarter",
        "Waxing Gibbous" to ".MainActivityWaxingGibbous",
        "Waning Gibbous" to ".MainActivityWaningGibbous",
        "Last Quarter" to ".MainActivityLastQuarter",
        "Waning Crescent" to ".MainActivityWaningCrescent",
        "New Moon" to ".MainActivityDefault"
    )

    fun updateIcon(context: Context, phaseName: String) {
        val targetAlias = phaseToAlias[phaseName] ?: return
        val currentPackage = context.packageName
        val pm = context.packageManager
        
        Log.d("MoonPhase", "Updating icon to: $targetAlias for phase: $phaseName")

        // Ensure the absolute target alias path
        val targetComponentName = ComponentName(currentPackage, "$currentPackage$targetAlias")

        // Check if the target is already enabled.
        if (pm.getComponentEnabledSetting(targetComponentName) == PackageManager.COMPONENT_ENABLED_STATE_ENABLED) {
            Log.d("MoonPhase", "Icon already set to $targetAlias")
            return
        }

        // Enable the new alias
        pm.setComponentEnabledSetting(
            targetComponentName,
            PackageManager.COMPONENT_ENABLED_STATE_ENABLED,
            0
        )

        // Disable all other aliases
        phaseToAlias.values.forEach { alias ->
            if (alias != targetAlias) {
                val comp = ComponentName(currentPackage, "$currentPackage$alias")
                pm.setComponentEnabledSetting(
                    comp,
                    PackageManager.COMPONENT_ENABLED_STATE_DISABLED,
                    0
                )
            }
        }
        Log.d("MoonPhase", "Icon swap triggered.")
    }
}
