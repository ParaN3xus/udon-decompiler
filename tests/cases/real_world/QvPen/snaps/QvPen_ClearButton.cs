// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.Udon.UI
{
    public class QvPen_ClearButton : UdonSharpBehaviour
    {
        System.Single targetTime2 = 0.0f;
        System.Single targetTime1 = 0.0f;
        VRC.Udon.Common.UdonInputEventArgs inputUseArgs = null /* {"eventType": 0, "boolValue": false, "floatValue": 0.0, "handType": 0} */;
        System.Int64[] __refl_typeids = null /* [3142330649835999805, -5351660574498133152] */;
        UnityEngine.UI.Image allTextImage = null;
        System.Boolean isPickedUp = false;
        UnityEngine.UI.Image allIndicator = null;
        System.Boolean isIn_LoopTextImageActive = false;
        UnityEngine.UI.Image ownIndicator = null;
        System.Boolean __1_isActive__param = false;
        System.Single __0_ownIndicatorValue__param = 0.0f;
        System.Boolean __0_isActive__param = false;
        System.Single targetTime_TextImage = 0.0f;
        System.Boolean inputUseBoolValue = false;
        System.Single __0_allIndicatorValue__param = 0.0f;
        System.Boolean isInInteract = false;
        UnityEngine.UI.Image ownTextImage = null;
        VRC.Udon.UdonBehaviour penManager = null;

        public void OnPenPickup()
        {
            isPickedUp = true;
            function_3();
            return;
        }

        public void OnPenDrop()
        {
            isPickedUp = false;
            targetTime_TextImage = UnityEngine.Time.time + 5.0f;
            function_3();
            return;
        }

        public void _start()
        {
            __0_isActive__param = false;
            function_0();
            __1_isActive__param = false;
            function_1();
            penManager.SetProgramVariable("__0_listener__param", this);
            penManager.SendCustomEvent("__0_Register");
            return;
        }

        public void _inputUse()
        {
            System.Boolean __intnl_SystemBoolean_0;
            if (inputUseBoolValue)
            {
                return;
            }
            else
            {
                __intnl_SystemBoolean_0 = isInInteract;
                if (__intnl_SystemBoolean_0)
                {
                    __intnl_SystemBoolean_0 = UnityEngine.Time.time < targetTime1;
                }
                if (__intnl_SystemBoolean_0)
                {
                    function_6();
                }
                isInInteract = false;
                __0_isActive__param = false;
                function_0();
                return;
            }
        }

        public void _interact()
        {
            isInInteract = true;
            __0_isActive__param = true;
            function_0();
            targetTime1 = UnityEngine.Time.time + 0.31f;
            targetTime2 = UnityEngine.Time.time + 2.0f;
            targetTime_TextImage = UnityEngine.Time.time + 5.0f;
            this.SendCustomEventDelayedSeconds("_LoopIndicator1", 0.0f, null /* 0 */);
            function_3();
            return;
        }

        void function_0()
        {
            UnityEngine.GameObject __intnl_UnityEngineGameObject_0;
            if (VRC.SDKBase.Utilities.IsValid(ownIndicator))
            {
                __intnl_UnityEngineGameObject_0 = ownIndicator.gameObject;
                __intnl_UnityEngineGameObject_0.SetActive(__0_isActive__param);
                ownIndicator.fillAmount = 0.0f;
            }
            if (VRC.SDKBase.Utilities.IsValid(allIndicator))
            {
                __intnl_UnityEngineGameObject_0 = allIndicator.gameObject;
                __intnl_UnityEngineGameObject_0.SetActive(__0_isActive__param);
                allIndicator.fillAmount = 0.0f;
            }
            return;
        }

        void function_1()
        {
            if (VRC.SDKBase.Utilities.IsValid(ownTextImage))
            {
                ownTextImage.gameObject.SetActive(__1_isActive__param);
            }
            if (VRC.SDKBase.Utilities.IsValid(allTextImage))
            {
                allTextImage.gameObject.SetActive(__1_isActive__param);
            }
            return;
        }

        void function_2()
        {
            if (VRC.SDKBase.Utilities.IsValid(ownIndicator))
            {
                ownIndicator.fillAmount = UnityEngine.Mathf.Clamp01(__0_ownIndicatorValue__param);
            }
            if (VRC.SDKBase.Utilities.IsValid(allIndicator))
            {
                allIndicator.fillAmount = UnityEngine.Mathf.Clamp01(__0_allIndicatorValue__param);
            }
            return;
        }

        public void _LoopIndicator1()
        {
            System.Single __intnl_SystemSingle_8;
            System.Single __intnl_SystemSingle_7;
            System.Single __lcl_leaveTime2_SystemSingle_0;
            System.Single __lcl_leaveTime1_SystemSingle_0;
            System.Single __lcl_time_SystemSingle_0;
            if (isInInteract)
            {
                __lcl_time_SystemSingle_0 = UnityEngine.Time.time;
                __lcl_leaveTime1_SystemSingle_0 = targetTime1 - __lcl_time_SystemSingle_0;
                __lcl_leaveTime2_SystemSingle_0 = targetTime2 - __lcl_time_SystemSingle_0;
                if (__lcl_leaveTime1_SystemSingle_0 <= 0.0f)
                {
                    function_5();
                    __intnl_SystemSingle_7 = __lcl_leaveTime2_SystemSingle_0 / 2.0f;
                    __intnl_SystemSingle_8 = 1.0f - __intnl_SystemSingle_7;
                    __0_ownIndicatorValue__param = 1.0f;
                    __0_allIndicatorValue__param = __intnl_SystemSingle_8;
                    function_2();
                    this.SendCustomEventDelayedFrames("_LoopIndicator2", 0, null /* 0 */);
                    return;
                }
                else
                {
                    __intnl_SystemSingle_7 = __lcl_leaveTime1_SystemSingle_0 / 0.31f;
                    __intnl_SystemSingle_8 = 1.0f - __intnl_SystemSingle_7;
                    __0_ownIndicatorValue__param = __intnl_SystemSingle_8;
                    __0_allIndicatorValue__param = 1.0f - __lcl_leaveTime2_SystemSingle_0 / 2.0f;
                    function_2();
                    this.SendCustomEventDelayedFrames("_LoopIndicator1", 0, null /* 0 */);
                    return;
                }
            }
            else
            {
                return;
            }
        }

        public void _LoopIndicator2()
        {
            System.Single __lcl_leaveTime2_SystemSingle_1;
            if (isInInteract)
            {
                __lcl_leaveTime2_SystemSingle_1 = targetTime2 - UnityEngine.Time.time;
                if (__lcl_leaveTime2_SystemSingle_1 <= 0.0f)
                {
                    function_7();
                    __0_ownIndicatorValue__param = 0.0f;
                    __0_allIndicatorValue__param = 0.0f;
                    function_2();
                    return;
                }
                else
                {
                    __0_ownIndicatorValue__param = 0.0f;
                    __0_allIndicatorValue__param = 1.0f - __lcl_leaveTime2_SystemSingle_1 / 2.0f;
                    function_2();
                    this.SendCustomEventDelayedFrames("_LoopIndicator2", 0, null /* 0 */);
                    return;
                }
            }
            else
            {
                return;
            }
        }

        void function_3()
        {
            if (isIn_LoopTextImageActive)
            {
                return;
            }
            else
            {
                isIn_LoopTextImageActive = true;
                if (isPickedUp)
                {
                    function_4();
                    return;
                }
                else
                {
                    __1_isActive__param = true;
                    function_1();
                    _LoopTextImageActive();
                    return;
                }
            }
        }

        void function_4()
        {
            if (isIn_LoopTextImageActive)
            {
                isIn_LoopTextImageActive = false;
                __1_isActive__param = false;
                function_1();
                return;
            }
            else
            {
                return;
            }
        }

        public void _LoopTextImageActive()
        {
            System.Single __lcl_leaveTime_TextImage_SystemSingle_0;
            System.Single __lcl_time_SystemSingle_1;
            if (isIn_LoopTextImageActive)
            {
                if (isPickedUp)
                {
                    function_4();
                    return;
                }
                else
                {
                    __lcl_time_SystemSingle_1 = UnityEngine.Time.time;
                    __lcl_leaveTime_TextImage_SystemSingle_0 = targetTime_TextImage - __lcl_time_SystemSingle_1;
                    if (__lcl_leaveTime_TextImage_SystemSingle_0 <= 0.0f)
                    {
                        function_4();
                        return;
                    }
                    else
                    {
                        this.SendCustomEventDelayedSeconds("_LoopTextImageActive", __lcl_leaveTime_TextImage_SystemSingle_0 / 2.0f, null /* 0 */);
                        return;
                    }
                }
            }
            else
            {
                return;
            }
        }

        void function_5()
        {
            if (VRC.SDKBase.Utilities.IsValid(penManager))
            {
                penManager.SendCustomEvent("EraseOwnInk");
            }
            return;
        }

        void function_6()
        {
            if (VRC.SDKBase.Utilities.IsValid(penManager))
            {
                penManager.SendCustomEvent("UndoDraw");
            }
            return;
        }

        void function_7()
        {
            if (VRC.SDKBase.Utilities.IsValid(penManager))
            {
                penManager.SendCustomNetworkEvent(null /* 0 */, "Clear");
            }
            return;
        }
    }
}