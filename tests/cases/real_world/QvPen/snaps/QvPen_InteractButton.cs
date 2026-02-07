// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.Udon.UI
{
    public class QvPen_InteractButton : UdonSharpBehaviour
    {
        System.Boolean canUseInstanceOwner = false;
        System.Boolean canUseEveryone = false;
        System.Boolean canUseOwner = false;
        System.Boolean canUseMaster = false;
        VRC.Udon.UdonBehaviour udonSharpBehaviour = null;
        UnityEngine.Component[] udonSharpBehaviours = null /* [] */;
        System.Boolean onlySendToOwner = false;
        System.String customEventName = "Unnamed";
        System.Boolean isGlobalEvent = false;

        public void _interact()
        {
            VRC.Udon.UdonBehaviour __lcl_udonSharpBehaviour_VRCUdonUdonBehaviour_0;
            System.String __intnl_SystemString_0;
            System.Int32 __intnl_SystemInt32_1;
            System.Int32 __intnl_SystemInt32_2;
            VRC.Udon.Common.Interfaces.NetworkEventTarget __intnl_VRCUdonCommonInterfacesNetworkEventTarget_0;
            System.Boolean __intnl_SystemBoolean_0;
            System.Boolean __intnl_SystemBoolean_1;
            System.Boolean __intnl_SystemBoolean_2;
            System.Boolean __intnl_SystemBoolean_3;
            System.Boolean __intnl_SystemBoolean_4;
            if (canUseEveryone)
            {
                goto label_bb_00000178;
            }
            else
            {
                __intnl_SystemBoolean_0 = canUseInstanceOwner;
                if (__intnl_SystemBoolean_0)
                {
                    __intnl_SystemBoolean_1 = VRC.SDKBase.Networking.IsInstanceOwner;
                    __intnl_SystemBoolean_0 = !__intnl_SystemBoolean_1;
                }
                if (__intnl_SystemBoolean_0)
                {
                    return;
                }
                else
                {
                    __intnl_SystemBoolean_2 = canUseMaster;
                    if (__intnl_SystemBoolean_2)
                    {
                        __intnl_SystemBoolean_3 = VRC.SDKBase.Networking.IsMaster;
                        __intnl_SystemBoolean_2 = !__intnl_SystemBoolean_3;
                    }
                    if (__intnl_SystemBoolean_2)
                    {
                        return;
                    }
                    else
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
                        else
                        {
                            goto label_bb_00000178;
                        }
                    }
                }
            }
        label_bb_00000178:
            __intnl_SystemBoolean_0 = VRC.SDKBase.Utilities.IsValid(udonSharpBehaviour);
            if (__intnl_SystemBoolean_0)
            {
                if (isGlobalEvent)
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
                    udonSharpBehaviour.SendCustomNetworkEvent(__intnl_VRCUdonCommonInterfacesNetworkEventTarget_0, customEventName);
                }
                else
                {
                    udonSharpBehaviour.SendCustomEvent(customEventName);
                }
            }
            __intnl_SystemBoolean_1 = udonSharpBehaviours.Length > 0;
            if (__intnl_SystemBoolean_1)
            {
                if (isGlobalEvent)
                {
                    __intnl_SystemInt32_1 = udonSharpBehaviours.Length;
                    __intnl_SystemInt32_2 = 0;
                    __intnl_SystemBoolean_2 = __intnl_SystemInt32_2 < __intnl_SystemInt32_1;
                    if (__intnl_SystemBoolean_2)
                    {
                        __lcl_udonSharpBehaviour_VRCUdonUdonBehaviour_0 = udonSharpBehaviours.Get(__intnl_SystemInt32_2);
                        __intnl_SystemBoolean_3 = VRC.SDKBase.Utilities.IsValid(__lcl_udonSharpBehaviour_VRCUdonUdonBehaviour_0);
                        if (__intnl_SystemBoolean_3)
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
                        goto label_bb_000002f0;
                    }
                    else
                    {
                        goto label_bb_00000504;
                    }
                }
                else
                {
                    __intnl_SystemInt32_1 = udonSharpBehaviours.Length;
                    __intnl_SystemInt32_2 = 0;
                    __intnl_SystemBoolean_2 = __intnl_SystemInt32_2 < __intnl_SystemInt32_1;
                    if (__intnl_SystemBoolean_2)
                    {
                        __lcl_udonSharpBehaviour_VRCUdonUdonBehaviour_0 = udonSharpBehaviours.Get(__intnl_SystemInt32_2);
                        __intnl_SystemBoolean_3 = VRC.SDKBase.Utilities.IsValid(__lcl_udonSharpBehaviour_VRCUdonUdonBehaviour_0);
                        if (__intnl_SystemBoolean_3)
                        {
                            __intnl_SystemString_0 = customEventName;
                            __lcl_udonSharpBehaviour_VRCUdonUdonBehaviour_0.SendCustomEvent(customEventName);
                        }
                        __intnl_SystemInt32_2 = __intnl_SystemInt32_2 + 1;
                        goto label_bb_00000438;
                    }
                    else
                    {
                        goto label_bb_00000504;
                    }
                }
            }
        label_bb_00000504:
            return;
        label_bb_000002f0:
        label_bb_00000438:
        }
    }
}