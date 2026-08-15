package com.urmanac.moonphase

import android.content.Context
import android.util.Log
import androidx.work.Constraints
import androidx.work.CoroutineWorker
import androidx.work.ExistingPeriodicWorkPolicy
import androidx.work.PeriodicWorkRequestBuilder
import androidx.work.WorkManager
import androidx.work.WorkerParameters
import org.json.JSONObject
import java.util.concurrent.TimeUnit

/**
 * CoroutineWorker that runs 4x-6x daily (every 4-6 hours) to calculate
 * the current moon phase from the WASM brain and swap the launcher app icon
 * without requiring the user to open the app.
 */
class LunarSyncWorker(
    appContext: Context,
    workerParams: WorkerParameters
) : CoroutineWorker(appContext, workerParams) {

    override suspend fun doWork(): Result {
        Log.d("MoonPhase", "LunarSyncWorker executing background moon phase calculation...")
        return try {
            val wasmEngine = WasmEngine(applicationContext.assets.open("moon-phase.wasm"))
            val now = System.currentTimeMillis() / 1000.0
            val result = wasmEngine.calculatePhase(now)
            val data = JSONObject(result)
            val phaseName = data.optString("phase_name")

            if (!phaseName.isNullOrEmpty()) {
                Log.d("MoonPhase", "LunarSyncWorker calculated phase: $phaseName")
                LunarIconManager.updateIcon(applicationContext, phaseName)
                Result.success()
            } else {
                Log.w("MoonPhase", "LunarSyncWorker: Empty phase name returned from WASM")
                Result.retry()
            }
        } catch (e: Exception) {
            Log.e("MoonPhase", "LunarSyncWorker failed", e)
            Result.retry()
        }
    }

    companion object {
        private const val UNIQUE_WORK_NAME = "lunar_icon_periodic_sync"

        /**
         * Schedules periodic background sync (every 6 hours / 4x daily).
         */
        fun schedule(context: Context) {
            val workRequest = PeriodicWorkRequestBuilder<LunarSyncWorker>(
                repeatInterval = 6,
                repeatIntervalTimeUnit = TimeUnit.HOURS
            ).build()

            WorkManager.getInstance(context).enqueueUniquePeriodicWork(
                UNIQUE_WORK_NAME,
                ExistingPeriodicWorkPolicy.KEEP,
                workRequest
            )
            Log.d("MoonPhase", "Scheduled periodic LunarSyncWorker every 6 hours")
        }
    }
}
