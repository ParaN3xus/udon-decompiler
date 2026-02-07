// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.UdonScript.UI
{
    public class QvPen_ToggleModeButton : UdonSharpBehaviour
    {
        System.Boolean isOn = false;
        UnityEngine.GameObject displayObjectOff = null;
        UnityEngine.GameObject displayObjectOn = null;
        VRC.Udon.UdonBehaviour settings = null;
        System.Int32 mode = 0;

        public void _start()
        {
            function_0();
            return;
        }

        public void _interact()
        {
            isOn = isOn ^ true;
            function_0();
            return;
        }

        void function_0()
        {
            System.Object __intnl_SystemObject_0;
            VRC.Udon.UdonBehaviour __lcl_penManager_VRCUdonUdonBehaviour_0;
            UnityEngine.Component[] __intnl_UnityEngineComponentArray_0;
            System.Int32 __intnl_SystemInt32_1;
            System.Int32 __intnl_SystemInt32_0;
            System.Boolean __intnl_SystemBoolean_4;
            System.Boolean __intnl_SystemBoolean_5;
            System.Boolean __intnl_SystemBoolean_6;
            System.Boolean __intnl_SystemBoolean_7;
            if (VRC.SDKBase.Utilities.IsValid(displayObjectOn))
            {
                displayObjectOn.SetActive(isOn);
            }
            if (VRC.SDKBase.Utilities.IsValid(displayObjectOff))
            {
                displayObjectOff.SetActive(!isOn);
            }
            if (!(mode == 1))
            {
                __intnl_SystemBoolean_4 = mode == 2;
                if (!__intnl_SystemBoolean_4)
                {
                    __intnl_SystemBoolean_5 = mode == 3;
                    if (__intnl_SystemBoolean_5)
                    {
                        __intnl_SystemObject_0 = settings.GetProgramVariable("penManagers");
                        __intnl_UnityEngineComponentArray_0 = __intnl_SystemObject_0;
                        __intnl_SystemInt32_0 = __intnl_UnityEngineComponentArray_0.Length;
                        __intnl_SystemInt32_1 = 0;
                        __intnl_SystemBoolean_6 = __intnl_SystemInt32_1 < __intnl_SystemInt32_0;
                        if (__intnl_SystemBoolean_6)
                        {
                            __lcl_penManager_VRCUdonUdonBehaviour_0 = __intnl_UnityEngineComponentArray_0.Get(__intnl_SystemInt32_1);
                            __intnl_SystemBoolean_7 = VRC.SDKBase.Utilities.IsValid(__lcl_penManager_VRCUdonUdonBehaviour_0);
                            if (__intnl_SystemBoolean_7)
                            {
                                __lcl_penManager_VRCUdonUdonBehaviour_0.SetProgramVariable("__2_value__param", isOn);
                                __lcl_penManager_VRCUdonUdonBehaviour_0.SendCustomEvent("__0__SetUsingSurftraceMode");
                            }
                            __intnl_SystemInt32_1 = __intnl_SystemInt32_1 + 1;
                            goto label_bb_000004b0;
                        }
                        else
                        {
                            goto label_bb_000005a4;
                        }
                    }
                }
                else
                {
                    __intnl_SystemObject_0 = settings.GetProgramVariable("penManagers");
                    __intnl_UnityEngineComponentArray_0 = __intnl_SystemObject_0;
                    __intnl_SystemInt32_0 = __intnl_UnityEngineComponentArray_0.Length;
                    __intnl_SystemInt32_1 = 0;
                    __intnl_SystemBoolean_5 = __intnl_SystemInt32_1 < __intnl_SystemInt32_0;
                    if (__intnl_SystemBoolean_5)
                    {
                        __lcl_penManager_VRCUdonUdonBehaviour_0 = __intnl_UnityEngineComponentArray_0.Get(__intnl_SystemInt32_1);
                        __intnl_SystemBoolean_6 = VRC.SDKBase.Utilities.IsValid(__lcl_penManager_VRCUdonUdonBehaviour_0);
                        if (__intnl_SystemBoolean_6)
                        {
                            __intnl_SystemBoolean_7 = isOn;
                            __lcl_penManager_VRCUdonUdonBehaviour_0.SetProgramVariable("__1_value__param", isOn);
                            __lcl_penManager_VRCUdonUdonBehaviour_0.SendCustomEvent("__0__SetEnabledLateSync");
                        }
                        __intnl_SystemInt32_1 = __intnl_SystemInt32_1 + 1;
                        goto label_bb_0000032c;
                    }
                    else
                    {
                        goto label_bb_000005a4;
                    }
                }
            }
            else
            {
                __intnl_SystemObject_0 = settings.GetProgramVariable("penManagers");
                __intnl_UnityEngineComponentArray_0 = __intnl_SystemObject_0;
                __intnl_SystemInt32_0 = __intnl_UnityEngineComponentArray_0.Length;
                __intnl_SystemInt32_1 = 0;
                __intnl_SystemBoolean_4 = __intnl_SystemInt32_1 < __intnl_SystemInt32_0;
                if (__intnl_SystemBoolean_4)
                {
                    __lcl_penManager_VRCUdonUdonBehaviour_0 = __intnl_UnityEngineComponentArray_0.Get(__intnl_SystemInt32_1);
                    __intnl_SystemBoolean_5 = VRC.SDKBase.Utilities.IsValid(__lcl_penManager_VRCUdonUdonBehaviour_0);
                    if (__intnl_SystemBoolean_5)
                    {
                        __intnl_SystemBoolean_6 = isOn;
                        __lcl_penManager_VRCUdonUdonBehaviour_0.SetProgramVariable("__0_value__param", isOn);
                        __lcl_penManager_VRCUdonUdonBehaviour_0.SendCustomEvent("__0__SetUsingDoubleClick");
                    }
                    __intnl_SystemInt32_1 = __intnl_SystemInt32_1 + 1;
                    goto label_bb_000001a8;
                }
                else
                {
                    goto label_bb_000005a4;
                }
            }
        label_bb_000005a4:
            return;
        label_bb_000001a8:
        label_bb_0000032c:
        label_bb_000004b0:
        }
    }
}