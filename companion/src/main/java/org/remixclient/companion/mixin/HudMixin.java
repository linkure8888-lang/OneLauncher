package org.remixclient.companion.mixin;

import net.minecraft.client.DeltaTracker;
import net.minecraft.client.gui.GuiGraphicsExtractor;
import net.minecraft.client.gui.Hud;
import org.remixclient.companion.RemixCompanionClient;
import org.remixclient.companion.RemixCrosshair;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;

/**
 * Hides the vanilla crosshair while the Remix crosshair module is on.
 *
 * <p>Vanilla draws the crosshair inside {@code Hud.extractCrosshair} with no Fabric event around
 * it, so cancelling the method is the only way to avoid two crosshairs on screen.</p>
 */
@Mixin(Hud.class)
public class HudMixin {

    @Inject(method = "extractCrosshair", at = @At("HEAD"), cancellable = true)
    private void remix$hideVanillaCrosshair(GuiGraphicsExtractor context, DeltaTracker delta,
                                            CallbackInfo callback) {
        if (!RemixCompanionClient.getConfig().showCrosshair) {
            return;
        }
        RemixCrosshair.markVanillaHidden();
        callback.cancel();
    }
}
