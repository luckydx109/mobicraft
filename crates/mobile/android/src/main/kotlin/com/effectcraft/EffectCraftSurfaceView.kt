package com.effectcraft

import android.content.Context
import android.util.AttributeSet
import android.view.SurfaceHolder
import android.view.SurfaceView

/**
 * Hardware-accelerated viewport for EffectCraft motion graphics presentation.
 */
class EffectCraftSurfaceView @JvmOverloads constructor(
    context: Context,
    attrs: AttributeSet? = null,
    defStyleAttr: Int = 0
) : SurfaceView(context, attrs, defStyleAttr), SurfaceHolder.Callback {

    init {
        holder.addCallback(this)
    }

    override fun surfaceCreated(holder: SurfaceHolder) {
        val width = width.coerceAtLeast(1)
        val height = height.coerceAtLeast(1)
        EffectCraftBridge.nativeSetSurface(holder.surface, width, height)
    }

    override fun surfaceChanged(holder: SurfaceHolder, format: Int, width: Int, height: Int) {
        EffectCraftBridge.nativeSetSurface(holder.surface, width.coerceAtLeast(1), height.coerceAtLeast(1))
    }

    override fun surfaceDestroyed(holder: SurfaceHolder) {
        EffectCraftBridge.nativeDestroySurface()
    }

    /**
     * Seek the timeline to a specific tick and render frame immediately.
     */
    fun seek(compId: Long = 0L, tick: Long, scale: Float = 0.5f) {
        EffectCraftBridge.nativeSeekFrame(compId, tick, scale)
    }
}
