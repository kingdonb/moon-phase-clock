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

        // Enable the new alias
        pm.setComponentEnabledSetting(
            targetComponentName,
            PackageManager.COMPONENT_ENABLED_STATE_ENABLED,
            PackageManager.DONT_KILL_APP
        )

        // Disable all other aliases
        var anyDisabled = false
        phaseToAlias.values.forEach { alias ->
            if (alias != targetAlias) {
                val comp = ComponentName(currentPackage, "$currentPackage$alias")
                if (pm.getComponentEnabledSetting(comp) != PackageManager.COMPONENT_ENABLED_STATE_DISABLED) {
                    pm.setComponentEnabledSetting(
                        comp,
                        PackageManager.COMPONENT_ENABLED_STATE_DISABLED,
                        PackageManager.DONT_KILL_APP
                    )
                    anyDisabled = true
                }
            }
        }
        
        if (anyDisabled) {
            Log.d("MoonPhase", "Icon swap completed (old aliases disabled).")
        } else {
            Log.d("MoonPhase", "Icon already correctly configured.")
        }
    }
}
