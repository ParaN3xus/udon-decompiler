// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.UdonScript.World
{
    public class QvPen_PlayerMods : UdonSharpBehaviour
    {
        System.Single jumpImpulse = 3.0f;
        VRC.SDKBase.VRCPlayerApi onPlayerJoinedPlayer = null;
        System.Boolean useLegacyLocomotion = false;
        System.Single runSpeed = 4.0f;
        System.Single walkSpeed = 2.0f;
        System.Single strafeSpeed = 2.0f;
        System.Single gravityStrength = 1.0f;

        public void _onPlayerJoined()
        {
            if (onPlayerJoinedPlayer.isLocal)
            {
                onPlayerJoinedPlayer.SetRunSpeed(runSpeed);
                onPlayerJoinedPlayer.SetWalkSpeed(walkSpeed);
                onPlayerJoinedPlayer.SetStrafeSpeed(strafeSpeed);
                onPlayerJoinedPlayer.SetJumpImpulse(jumpImpulse);
                onPlayerJoinedPlayer.SetGravityStrength(gravityStrength);
                if (useLegacyLocomotion)
                {
                    onPlayerJoinedPlayer.UseLegacyLocomotion();
                }
            }
            return;
        }
    }
}