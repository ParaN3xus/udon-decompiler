// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.UdonScript
{
    public class QvPen_Settings : UdonSharpBehaviour
    {
        System.String version = "";
        UnityEngine.TextAsset versionText = null;
        UnityEngine.UI.Text information = null;
        TMPro.TextMeshPro informationTMP = null;
        TMPro.TextMeshProUGUI informationTMPU = null;
        UnityEngine.Transform pensParent = null;
        UnityEngine.Transform erasersParent = null;
        UnityEngine.Component[] penManagers = null /* [] */;
        UnityEngine.Component[] eraserManagers = null /* [] */;
        UnityEngine.Component[] __0__intnlparam = null;
        UnityEngine.Component __1__intnlparam = null;
        UnityEngine.Component[] __2__intnlparam = null;
        UnityEngine.Component __3__intnlparam = null;
        UnityEngine.Component[] __6__intnlparam = null;
        UnityEngine.Component[] __7__intnlparam = null;
        UnityEngine.Component[] __8__intnlparam = null;
        UnityEngine.Component[] __9__intnlparam = null;

        public void _start()
        {
            System.String __lcl_informationText_SystemString_0 = null;

            if (VRC.SDKBase.Utilities.IsValid(versionText))
            {
                version = versionText.text.Trim();
            }
            __lcl_informationText_SystemString_0 = "<size=20></size>\n" + System.String.Format("<size=14>{0}</size>", version);
            if (VRC.SDKBase.Utilities.IsValid(information))
            {
                information.text = __lcl_informationText_SystemString_0;
            }
            if (VRC.SDKBase.Utilities.IsValid(informationTMP))
            {
                informationTMP.text = __lcl_informationText_SystemString_0;
            }
            if (VRC.SDKBase.Utilities.IsValid(informationTMPU))
            {
                informationTMPU.text = __lcl_informationText_SystemString_0;
            }
            if (VRC.SDKBase.Utilities.IsValid(pensParent))
            {
                __1__intnlparam = pensParent.transform;
                function_0();
                penManagers = __0__intnlparam;
            }
            if (VRC.SDKBase.Utilities.IsValid(erasersParent))
            {
                __3__intnlparam = erasersParent.transform;
                function_1();
                eraserManagers = __2__intnlparam;
            }
            return;
        }

        void function_0()
        {
            UnityEngine.Component[] __lcl_instanceBehaviours_UnityEngineComponentArray_0 = null;

            __lcl_instanceBehaviours_UnityEngineComponentArray_0 =
                __1__intnlparam.GetComponentsInChildren(null /* "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
            __7__intnlparam = __lcl_instanceBehaviours_UnityEngineComponentArray_0;
            function_2();
            __0__intnlparam = __6__intnlparam;
            return;
        }

        void function_1()
        {
            UnityEngine.Component[] __lcl_instanceBehaviours_UnityEngineComponentArray_1 = null;

            __lcl_instanceBehaviours_UnityEngineComponentArray_1 =
                __3__intnlparam.GetComponentsInChildren(null /* "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
            __9__intnlparam = __lcl_instanceBehaviours_UnityEngineComponentArray_1;
            function_3();
            __2__intnlparam = __8__intnlparam;
            return;
        }

        void function_2()
        {
            System.Int64 __lcl_targetID_SystemInt64_0 = 0;
            System.Int32 __lcl_arraySize_SystemInt32_0 = 0;
            UnityEngine.Component[] __lcl_foundBehaviours_UnityEngineComponentArray_0 = null;
            System.Int32 __lcl_targetIdx_SystemInt32_0 = 0;
            System.Int32 __intnl_SystemInt32_3 = 0;
            System.Int32 __intnl_SystemInt32_4 = 0;
            VRC.Udon.UdonBehaviour __lcl_behaviour_VRCUdonUdonBehaviour_0 = null;
            System.Object __lcl_typeID_SystemObject_0 = null;
            System.Boolean __intnl_SystemBoolean_10 = false;
            System.Int32 __intnl_SystemInt32_5 = 0;

            __lcl_targetID_SystemInt64_0 = -1598075768554875822;
            __lcl_arraySize_SystemInt32_0 = 0;
            __intnl_SystemInt32_3 = __7__intnlparam.Length;
            __intnl_SystemInt32_4 = 0;
            while (__intnl_SystemInt32_4 < __intnl_SystemInt32_3)
            {
                __lcl_behaviour_VRCUdonUdonBehaviour_0 = __7__intnlparam.Get(__intnl_SystemInt32_4);
                if (!(__lcl_behaviour_VRCUdonUdonBehaviour_0.GetProgramVariableType("__refl_typeid") == null))
                {
                    __lcl_typeID_SystemObject_0 = __lcl_behaviour_VRCUdonUdonBehaviour_0.GetProgramVariable("__refl_typeid");
                    __intnl_SystemBoolean_10 = __lcl_typeID_SystemObject_0 != null;
                    if (__intnl_SystemBoolean_10)
                    {
                        __intnl_SystemBoolean_10 = System.Convert.ToInt64(__lcl_typeID_SystemObject_0) == __lcl_targetID_SystemInt64_0;
                    }
                    if (__intnl_SystemBoolean_10)
                    {
                        __lcl_arraySize_SystemInt32_0 = __lcl_arraySize_SystemInt32_0 + 1;
                    }
                }
                __intnl_SystemInt32_4 = __intnl_SystemInt32_4 + 1;
            }
            __lcl_foundBehaviours_UnityEngineComponentArray_0 = new UnityEngine.Component[](__lcl_arraySize_SystemInt32_0);
            __lcl_targetIdx_SystemInt32_0 = 0;
            __intnl_SystemInt32_3 = __7__intnlparam.Length;
            __intnl_SystemInt32_4 = 0;
            while (__intnl_SystemInt32_4 < __intnl_SystemInt32_3)
            {
                __lcl_behaviour_VRCUdonUdonBehaviour_0 = __7__intnlparam.Get(__intnl_SystemInt32_4);
                if (!(__lcl_behaviour_VRCUdonUdonBehaviour_0.GetProgramVariableType("__refl_typeid") == null))
                {
                    __lcl_typeID_SystemObject_0 = __lcl_behaviour_VRCUdonUdonBehaviour_0.GetProgramVariable("__refl_typeid");
                    __intnl_SystemBoolean_10 = __lcl_typeID_SystemObject_0 != null;
                    if (__intnl_SystemBoolean_10)
                    {
                        __intnl_SystemBoolean_10 = System.Convert.ToInt64(__lcl_typeID_SystemObject_0) == __lcl_targetID_SystemInt64_0;
                    }
                    if (__intnl_SystemBoolean_10)
                    {
                        __intnl_SystemInt32_5 = __lcl_targetIdx_SystemInt32_0;
                        __lcl_targetIdx_SystemInt32_0 = __intnl_SystemInt32_5 + 1;
                        __lcl_foundBehaviours_UnityEngineComponentArray_0.Set(__intnl_SystemInt32_5, __lcl_behaviour_VRCUdonUdonBehaviour_0);
                    }
                }
                __intnl_SystemInt32_4 = __intnl_SystemInt32_4 + 1;
            }
            __6__intnlparam = __lcl_foundBehaviours_UnityEngineComponentArray_0;
            return;
        }

        void function_3()
        {
            System.Int64 __lcl_targetID_SystemInt64_1 = 0;
            System.Int32 __lcl_arraySize_SystemInt32_1 = 0;
            UnityEngine.Component[] __lcl_foundBehaviours_UnityEngineComponentArray_1 = null;
            System.Int32 __lcl_targetIdx_SystemInt32_1 = 0;
            System.Int32 __intnl_SystemInt32_6 = 0;
            System.Int32 __intnl_SystemInt32_7 = 0;
            VRC.Udon.UdonBehaviour __lcl_behaviour_VRCUdonUdonBehaviour_1 = null;
            System.Object __lcl_typeID_SystemObject_1 = null;
            System.Boolean __intnl_SystemBoolean_13 = false;
            System.Int32 __intnl_SystemInt32_8 = 0;

            __lcl_targetID_SystemInt64_1 = -5465713458096539185;
            __lcl_arraySize_SystemInt32_1 = 0;
            __intnl_SystemInt32_6 = __9__intnlparam.Length;
            __intnl_SystemInt32_7 = 0;
            while (__intnl_SystemInt32_7 < __intnl_SystemInt32_6)
            {
                __lcl_behaviour_VRCUdonUdonBehaviour_1 = __9__intnlparam.Get(__intnl_SystemInt32_7);
                if (!(__lcl_behaviour_VRCUdonUdonBehaviour_1.GetProgramVariableType("__refl_typeid") == null))
                {
                    __lcl_typeID_SystemObject_1 = __lcl_behaviour_VRCUdonUdonBehaviour_1.GetProgramVariable("__refl_typeid");
                    __intnl_SystemBoolean_13 = __lcl_typeID_SystemObject_1 != null;
                    if (__intnl_SystemBoolean_13)
                    {
                        __intnl_SystemBoolean_13 = System.Convert.ToInt64(__lcl_typeID_SystemObject_1) == __lcl_targetID_SystemInt64_1;
                    }
                    if (__intnl_SystemBoolean_13)
                    {
                        __lcl_arraySize_SystemInt32_1 = __lcl_arraySize_SystemInt32_1 + 1;
                    }
                }
                __intnl_SystemInt32_7 = __intnl_SystemInt32_7 + 1;
            }
            __lcl_foundBehaviours_UnityEngineComponentArray_1 = new UnityEngine.Component[](__lcl_arraySize_SystemInt32_1);
            __lcl_targetIdx_SystemInt32_1 = 0;
            __intnl_SystemInt32_6 = __9__intnlparam.Length;
            __intnl_SystemInt32_7 = 0;
            while (__intnl_SystemInt32_7 < __intnl_SystemInt32_6)
            {
                __lcl_behaviour_VRCUdonUdonBehaviour_1 = __9__intnlparam.Get(__intnl_SystemInt32_7);
                if (!(__lcl_behaviour_VRCUdonUdonBehaviour_1.GetProgramVariableType("__refl_typeid") == null))
                {
                    __lcl_typeID_SystemObject_1 = __lcl_behaviour_VRCUdonUdonBehaviour_1.GetProgramVariable("__refl_typeid");
                    __intnl_SystemBoolean_13 = __lcl_typeID_SystemObject_1 != null;
                    if (__intnl_SystemBoolean_13)
                    {
                        __intnl_SystemBoolean_13 = System.Convert.ToInt64(__lcl_typeID_SystemObject_1) == __lcl_targetID_SystemInt64_1;
                    }
                    if (__intnl_SystemBoolean_13)
                    {
                        __intnl_SystemInt32_8 = __lcl_targetIdx_SystemInt32_1;
                        __lcl_targetIdx_SystemInt32_1 = __intnl_SystemInt32_8 + 1;
                        __lcl_foundBehaviours_UnityEngineComponentArray_1.Set(__intnl_SystemInt32_8, __lcl_behaviour_VRCUdonUdonBehaviour_1);
                    }
                }
                __intnl_SystemInt32_7 = __intnl_SystemInt32_7 + 1;
            }
            __8__intnlparam = __lcl_foundBehaviours_UnityEngineComponentArray_1;
            return;
        }
    }
}