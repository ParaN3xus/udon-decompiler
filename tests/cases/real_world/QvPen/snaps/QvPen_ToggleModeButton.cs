// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.UdonScript.UI
{
    public class QvPen_ToggleModeButton : UdonSharpBehaviour
    {
        VRC.Udon.UdonBehaviour settings = null;
        System.Int32 mode = 0;
        System.Boolean isOn = false;
        UnityEngine.GameObject displayObjectOn = null;
        UnityEngine.GameObject displayObjectOff = null;

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
            UnityEngine.Component[] __intnl_UnityEngineComponentArray_0 = null;
            System.Int32 __intnl_SystemInt32_0 = 0;
            System.Int32 __intnl_SystemInt32_1 = 0;
            VRC.Udon.UdonBehaviour __lcl_penManager_VRCUdonUdonBehaviour_0 = null;
            System.Boolean __intnl_SystemBoolean_6 = false;
            System.Boolean __intnl_SystemBoolean_7 = false;
            System.Boolean __intnl_SystemBoolean_8 = false;

            if (VRC.SDKBase.Utilities.IsValid(displayObjectOn))
            {
                displayObjectOn.SetActive(isOn);
            }
            if (VRC.SDKBase.Utilities.IsValid(displayObjectOff))
            {
                displayObjectOff.SetActive(!isOn);
            }
            if (mode == 1)
            {
                __intnl_UnityEngineComponentArray_0 = settings.GetProgramVariable("penManagers");
                __intnl_SystemInt32_0 = __intnl_UnityEngineComponentArray_0.Length;
                __intnl_SystemInt32_1 = 0;
                while (__intnl_SystemInt32_1 < __intnl_SystemInt32_0)
                {
                    __lcl_penManager_VRCUdonUdonBehaviour_0 = __intnl_UnityEngineComponentArray_0.Get(__intnl_SystemInt32_1);
                    if (VRC.SDKBase.Utilities.IsValid(__lcl_penManager_VRCUdonUdonBehaviour_0))
                    {
                        __intnl_SystemBoolean_6 = isOn;
                        __lcl_penManager_VRCUdonUdonBehaviour_0.SetProgramVariable("__0_value__param", isOn);
                        __lcl_penManager_VRCUdonUdonBehaviour_0.SendCustomEvent("__0__SetUsingDoubleClick");
                    }
                    __intnl_SystemInt32_1 = __intnl_SystemInt32_1 + 1;
                }
            }
            else
            {
                if (mode == 2)
                {
                    __intnl_UnityEngineComponentArray_0 = settings.GetProgramVariable("penManagers");
                    __intnl_SystemInt32_0 = __intnl_UnityEngineComponentArray_0.Length;
                    __intnl_SystemInt32_1 = 0;
                    while (__intnl_SystemInt32_1 < __intnl_SystemInt32_0)
                    {
                        __lcl_penManager_VRCUdonUdonBehaviour_0 = __intnl_UnityEngineComponentArray_0.Get(__intnl_SystemInt32_1);
                        if (VRC.SDKBase.Utilities.IsValid(__lcl_penManager_VRCUdonUdonBehaviour_0))
                        {
                            __intnl_SystemBoolean_7 = isOn;
                            __lcl_penManager_VRCUdonUdonBehaviour_0.SetProgramVariable("__1_value__param", isOn);
                            __lcl_penManager_VRCUdonUdonBehaviour_0.SendCustomEvent("__0__SetEnabledLateSync");
                        }
                        __intnl_SystemInt32_1 = __intnl_SystemInt32_1 + 1;
                    }
                }
                else
                {
                    if (mode == 3)
                    {
                        __intnl_UnityEngineComponentArray_0 = settings.GetProgramVariable("penManagers");
                        __intnl_SystemInt32_0 = __intnl_UnityEngineComponentArray_0.Length;
                        __intnl_SystemInt32_1 = 0;
                        while (__intnl_SystemInt32_1 < __intnl_SystemInt32_0)
                        {
                            __lcl_penManager_VRCUdonUdonBehaviour_0 = __intnl_UnityEngineComponentArray_0.Get(__intnl_SystemInt32_1);
                            if (VRC.SDKBase.Utilities.IsValid(__lcl_penManager_VRCUdonUdonBehaviour_0))
                            {
                                __intnl_SystemBoolean_8 = isOn;
                                __lcl_penManager_VRCUdonUdonBehaviour_0.SetProgramVariable("__2_value__param", isOn);
                                __lcl_penManager_VRCUdonUdonBehaviour_0.SendCustomEvent("__0__SetUsingSurftraceMode");
                            }
                            __intnl_SystemInt32_1 = __intnl_SystemInt32_1 + 1;
                        }
                    }
                }
            }
            return;
        }
    }
}