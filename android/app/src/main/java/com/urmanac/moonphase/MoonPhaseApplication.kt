package com.urmanac.moonphase

import android.app.Application

class MoonPhaseApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        // Ensure background periodic sync is armed on process startup
        LunarSyncWorker.schedule(this)
    }
}
