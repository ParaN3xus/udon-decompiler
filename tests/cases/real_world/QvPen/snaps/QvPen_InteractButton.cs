// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.Udon.UI
{
    public class QvPen_InteractButton : UdonSharpBehaviour
    {
        System.Boolean canUseEveryone = false;
        System.Boolean canUseInstanceOwner = false;
        System.Boolean canUseOwner = false;
        System.Boolean canUseMaster = false;
        System.Boolean isGlobalEvent = false;
        System.Boolean onlySendToOwner = false;
        VRC.Udon.UdonBehaviour udonSharpBehaviour = null;
        UnityEngine.Component[] udonSharpBehaviours = null /* [] */;
        System.String customEventName = "Unnamed";

        public void _interact()
        {
            System.Boolean __intnl_SystemBoolean_0 = false;
            System.Boolean __intnl_SystemBoolean_2 = false;
            System.Boolean __intnl_SystemBoolean_4 = false;
            VRC.Udon.Common.Interfaces.NetworkEventTarget __intnl_VRCUdonCommonInterfacesNetworkEventTarget_0 = null /* 0 */;
            VRC.Udon.UdonBehaviour __intnl_VRCUdonUdonBehaviour_0 = null;
            System.String __intnl_SystemString_0 = null;
            System.String __intnl_SystemString_1 = null;
            System.Int32 __intnl_SystemInt32_1 = 0;
            System.Int32 __intnl_SystemInt32_2 = 0;
            VRC.Udon.UdonBehaviour __lcl_udonSharpBehaviour_VRCUdonUdonBehaviour_0 = null;

            if (!canUseEveryone)
            {
                __intnl_SystemBoolean_0 = canUseInstanceOwner;
                if (__intnl_SystemBoolean_0)
                {
                    __intnl_SystemBoolean_0 = !VRC.SDKBase.Networking.IsInstanceOwner;
                }
                if (!__intnl_SystemBoolean_0)
                {
                    __intnl_SystemBoolean_2 = canUseMaster;
                    if (__intnl_SystemBoolean_2)
                    {
                        __intnl_SystemBoolean_2 = !VRC.SDKBase.Networking.IsMaster;
                    }
                    if (!__intnl_SystemBoolean_2)
                    {
                        __intnl_SystemBoolean_4 = canUseOwner;
                        if (__intnl_SystemBoolean_4)
                        {
                            __intnl_SystemBoolean_4 = !VRC.SDKBase.Networking.IsOwner(this.gameObject);
                        }
                        if (__intnl_SystemBoolean_4)
                        {
                            return;
                        }
                        goto label_bb_00000178;
                    }
                }
                return;
            }
        label_bb_00000178:
            if (VRC.SDKBase.Utilities.IsValid(udonSharpBehaviour))
            {
                if (isGlobalEvent)
                {
                    __intnl_VRCUdonUdonBehaviour_0 = udonSharpBehaviour;
                    if (onlySendToOwner)
                    {
                        __intnl_VRCUdonCommonInterfacesNetworkEventTarget_0 = null /* 1 */;
                    }
                    else
                    {
                        __intnl_VRCUdonCommonInterfacesNetworkEventTarget_0 = null /* 0 */;
                    }
                    __intnl_SystemString_0 = customEventName;
                    __intnl_VRCUdonUdonBehaviour_0.SendCustomNetworkEvent(__intnl_VRCUdonCommonInterfacesNetworkEventTarget_0, customEventName);
                }
                else
                {
                    __intnl_SystemString_1 = customEventName;
                    udonSharpBehaviour.SendCustomEvent(customEventName);
                }
            }
            if (udonSharpBehaviours.Length > 0)
            {
                if (isGlobalEvent)
                {
                    __intnl_SystemInt32_1 = udonSharpBehaviours.Length;
                    __intnl_SystemInt32_2 = 0;
                    while (__intnl_SystemInt32_2 < __intnl_SystemInt32_1)
                    {
                        __lcl_udonSharpBehaviour_VRCUdonUdonBehaviour_0 = udonSharpBehaviours.Get(__intnl_SystemInt32_2);
                        if (VRC.SDKBase.Utilities.IsValid(__lcl_udonSharpBehaviour_VRCUdonUdonBehaviour_0))
                        {
                            if (onlySendToOwner)
                            {
                                __intnl_VRCUdonCommonInterfacesNetworkEventTarget_0 = null /* 1 */;
                            }
                            else
                            {
                                __intnl_VRCUdonCommonInterfacesNetworkEventTarget_0 = null /* 0 */;
                            }
                            __intnl_SystemString_0 = customEventName;
                            __lcl_udonSharpBehaviour_VRCUdonUdonBehaviour_0.SendCustomNetworkEvent(__intnl_VRCUdonCommonInterfacesNetworkEventTarget_0,
                                                                                                   customEventName);
                        }
                        __intnl_SystemInt32_2 = __intnl_SystemInt32_2 + 1;
                    }
                }
                else
                {
                    __intnl_SystemInt32_1 = udonSharpBehaviours.Length;
                    __intnl_SystemInt32_2 = 0;
                    while (__intnl_SystemInt32_2 < __intnl_SystemInt32_1)
                    {
                        __lcl_udonSharpBehaviour_VRCUdonUdonBehaviour_0 = udonSharpBehaviours.Get(__intnl_SystemInt32_2);
                        if (VRC.SDKBase.Utilities.IsValid(__lcl_udonSharpBehaviour_VRCUdonUdonBehaviour_0))
                        {
                            __intnl_SystemString_0 = customEventName;
                            __lcl_udonSharpBehaviour_VRCUdonUdonBehaviour_0.SendCustomEvent(customEventName);
                        }
                        __intnl_SystemInt32_2 = __intnl_SystemInt32_2 + 1;
                    }
                }
            }
            return;
        }
    }
}