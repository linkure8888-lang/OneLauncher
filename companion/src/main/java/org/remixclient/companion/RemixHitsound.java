package org.remixclient.companion;

import net.minecraft.client.Minecraft;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.sounds.SoundEvents;
import net.minecraft.world.entity.LivingEntity;
import net.minecraft.world.phys.EntityHitResult;
import net.minecraft.world.phys.HitResult;

/**
 * Hit sound: a short chime the moment a swing connects with a living entity.
 *
 * <p>Played through the client's own sound engine as a UI sound, so nothing is sent to the
 * server and other players never hear it.</p>
 */
final class RemixHitsound {

    /** Base pitch; a critical hit sits slightly higher so the ear can tell them apart. */
    private static final float BASE_PITCH = 1.45f;
    private static final float CRIT_PITCH = 1.65f;

    private RemixHitsound() {
    }

    /**
     * Called before the local player attacks, right after the hit particles. The same attack
     * strength filter keeps a spam-clicker from stacking one chime per click instead of one
     * per real swing.
     */
    static void onAttack(Minecraft client, LocalPlayer player, RemixConfig config) {
        if (!config.hitsound || client.level == null || player == null) {
            return;
        }
        if (player.getAttackStrengthScale(0.5f) < 0.9f) {
            return;
        }

        HitResult hit = client.hitResult;
        if (!(hit instanceof EntityHitResult entityHit)
                || !(entityHit.getEntity() instanceof LivingEntity)) {
            return;
        }

        boolean crit = player.fallDistance > 0.0 && !player.onGround();
        float volume = Math.max(0f, Math.min(100f, config.hitsoundVolume)) / 100f;
        if (volume <= 0f) {
            return;
        }
        client.getSoundManager()
                .play(net.minecraft.client.resources.sounds.SimpleSoundInstance
                        .forUI(SoundEvents.EXPERIENCE_ORB_PICKUP, crit ? CRIT_PITCH : BASE_PITCH,
                                volume));
    }
}
