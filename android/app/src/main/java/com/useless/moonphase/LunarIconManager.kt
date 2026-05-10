package com.useless.moonphase

import android.content.ComponentName
import android.content.Context
import android.content.pm.PackageManager

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
        "New Moon" to "" // Default MainActivity
    )

    fun updateIcon(context: Context, phaseName: String) {
        val targetAlias = phaseToAlias[phaseName] ?: return
        val currentPackage = context.packageName
        
        val pm = context.packageManager
        
        // List of all possible aliases (including the default one)
        val allComponents = phaseToAlias.values.map { 
            if (it.isEmpty()) ".MainActivity" else it
        }

        allComponents.forEach { alias ->
            val componentName = ComponentName(currentPackage, "$currentPackage$alias")
            val newState = if (alias == (if (targetAlias.isEmpty()) ".MainActivity" else targetAlias)) {
                PackageManager.COMPONENT_ENABLED_STATE_ENABLED
            } else {
                PackageManager.COMPONENT_ENABLED_STATE_DISABLED
            }
            
            // Only update if the state has changed to avoid unnecessary churn
            if (pm.getComponentEnabledSetting(componentName) != newState) {
                pm.setComponentEnabledSetting(
                    componentName,
                    newState,
                    PackageManager.DONT_KILL_APP
                )
            }
        }
    }
}
