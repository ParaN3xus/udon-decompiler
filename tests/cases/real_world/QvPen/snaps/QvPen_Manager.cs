// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.UdonScript
{
    public class QvPen_Manager : UdonSharpBehaviour
    {
        System.Int32 __1_penId__param = 0;
        UnityEngine.Vector3 __6__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Color __8__intnlparam = null /* "RGBA(0.000, 0.000, 0.000, 0.000)" */;
        VRC.Udon.UdonBehaviour lastUsedPen = null;
        UnityEngine.Vector3 __0_ownerIdVector__param = null /* "(0.00, 0.00, 0.00)" */;
        System.String __0___0_ColorBeginTag__ret = null;
        System.Int32 __1_inkId__param = 0;
        System.Boolean __0___0_RemoveInk__ret = false;
        System.String __0_get_logPrefix__ret = null;
        System.Boolean __0___0_RemoveUserInk__ret = false;
        UnityEngine.GameObject __3__intnlparam = null;
        VRC.SDK3.Data.DataList callablePenList = null /* [] */;
        System.Int32 __5_penId__param = 0;
        VRC.Udon.UdonBehaviour __0_GetPen__ret = null;
        UnityEngine.Vector3 __const_UnityEngineVector3_0 = null /* "(0.00, 0.00, 1.00)" */;
        System.Int32 __0_penId__param = 0;
        System.Int32 __0__intnlparam = 0;
        System.String __7__intnlparam = null;
        System.String _logPrefix = null;
        System.Int32 __0_inkId__param = 0;
        VRC.SDK3.Data.DataDictionary inkDictMap = null /* [] */;
        UnityEngine.Vector3 __4__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        System.Boolean __0___0_HasInk__ret = false;
        System.Int32 __4_penId__param = 0;
        System.Int32 __3_penId__param = 0;
        UnityEngine.Vector3 __1__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector3 __const_UnityEngineVector3_1 = null /* "(0.00, 0.00, 0.00)" */;
        System.Object[] __gintnl_SystemObjectArray_0 = null /* [null, null, null, null, null] */;
        UnityEngine.GameObject __0_inkInstance__param = null;
        System.Object __1_o__param = null;
        System.Object __0_o__param = null;
        UnityEngine.Color __0_c__param = null /* "RGBA(0.000, 0.000, 0.000, 0.000)" */;
        System.Object __2_o__param = null;
        UnityEngine.Vector3 __5__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        VRC.SDK3.Data.DataDictionary penDict = null /* [] */;
        UnityEngine.Color logColor = null /* "RGBA(0.949, 0.490, 0.290, 1.000)" */;
        System.Int32 __2_penId__param = 0;
        System.Boolean __2__intnlparam = false;
        VRC.Udon.UdonBehaviour __1_pen__param = null;
        VRC.Udon.UdonBehaviour __0_pen__param = null;
        System.Int32 __2_inkId__param = 0;

        public void __0_Register()
        {
            if (penDict.ContainsKey((VRC.SDK3.Data.DataToken)__0_penId__param))
            {
                return;
            }
            else
            {
                penDict.set_Item((VRC.SDK3.Data.DataToken)__0_penId__param, (VRC.SDK3.Data.DataToken)__0_pen__param);
                inkDictMap.set_Item((VRC.SDK3.Data.DataToken)__0_penId__param, (VRC.SDK3.Data.DataToken) new VRC.SDK3.Data.DataDictionary());
                __0_pen__param.SendCustomEvent("get_AllowCallPen");
                if (__0_pen__param.GetProgramVariable("__0_get_AllowCallPen__ret"))
                {
                    callablePenList.Add((VRC.SDK3.Data.DataToken)__0_pen__param);
                }
                return;
            }
        }

        public void _update()
        {
            System.Boolean __intnl_SystemBoolean_3;
            if (UnityEngine.Input.anyKeyDown)
            {
                __intnl_SystemBoolean_3 = UnityEngine.Input.GetKeyDown(null /* 113 */);
                if (__intnl_SystemBoolean_3)
                {
                    __intnl_SystemBoolean_3 = UnityEngine.Input.GetKey(null /* 9 */);
                }
                if (__intnl_SystemBoolean_3)
                {
                    function_0();
                }
                return;
            }
            else
            {
                return;
            }
        }

        public void __0_SetLastUsedPen()
        {
            lastUsedPen = __1_pen__param;
            return;
        }

        void function_0()
        {
            UnityEngine.Vector3 __lcl_forward_UnityEngineVector3_0;
            VRC.SDKBase.VRCPlayerApi + TrackingData __lcl_x_VRCSDKBaseVRCPlayerApiTrackingData_0;
            VRC.Udon.UdonBehaviour __lcl_pen_VRCUdonUdonBehaviour_0;
            function_1();
            __lcl_pen_VRCUdonUdonBehaviour_0 = __0_GetPen__ret;
            if (VRC.SDKBase.Utilities.IsValid(__lcl_pen_VRCUdonUdonBehaviour_0))
            {
                __lcl_x_VRCSDKBaseVRCPlayerApiTrackingData_0 = VRC.SDKBase.Networking.LocalPlayer.GetTrackingData(null /* 0 */);
                __lcl_forward_UnityEngineVector3_0 = __lcl_x_VRCSDKBaseVRCPlayerApiTrackingData_0.rotation * __const_UnityEngineVector3_0;
                __lcl_pen_VRCUdonUdonBehaviour_0.transform.position =
                    __lcl_x_VRCSDKBaseVRCPlayerApiTrackingData_0.position + 0.5f * __lcl_forward_UnityEngineVector3_0;
                __lcl_pen_VRCUdonUdonBehaviour_0.transform.LookAt(__lcl_x_VRCSDKBaseVRCPlayerApiTrackingData_0.position + __lcl_forward_UnityEngineVector3_0);
                return;
            }
            else
            {
                return;
            }
        }

        void function_1()
        {
            System.Int32[] __lcl_indexList_SystemInt32Array_0;
            System.Int32 __lcl_index_SystemInt32_0;
            VRC.Udon.UdonBehaviour __lcl_pen_VRCUdonUdonBehaviour_1;
            VRC.SDK3.Components.VRCPickup __lcl_pickupR_VRCSDK3ComponentsVRCPickup_0;
            System.Boolean __intnl_SystemBoolean_11;
            System.Int32 __lcl_i_SystemInt32_0;
            VRC.SDK3.Components.VRCPickup __lcl_pickupL_VRCSDK3ComponentsVRCPickup_0;
            VRC.SDK3.Data.DataToken __lcl_penToken_VRCSDK3DataDataToken_0;
            System.Boolean __intnl_SystemBoolean_12;
            System.Object __intnl_SystemObject_3;
            System.Int32 __lcl_n_SystemInt32_0;
            System.Boolean __intnl_SystemBoolean_8;
            System.Boolean __intnl_SystemBoolean_7;
            __lcl_pickupR_VRCSDK3ComponentsVRCPickup_0 = VRC.SDKBase.Networking.LocalPlayer.GetPickupInHand(null /* 2 */);
            if (VRC.SDKBase.Utilities.IsValid(__lcl_pickupR_VRCSDK3ComponentsVRCPickup_0))
            {
                __0_GetPen__ret = null;
                return;
            }
            else
            {
                __lcl_pickupL_VRCSDK3ComponentsVRCPickup_0 = VRC.SDKBase.Networking.LocalPlayer.GetPickupInHand(null /* 1 */);
                if (VRC.SDKBase.Utilities.IsValid(__lcl_pickupL_VRCSDK3ComponentsVRCPickup_0))
                {
                    __0_GetPen__ret = null;
                    return;
                }
                else
                {
                    __intnl_SystemBoolean_8 = VRC.SDKBase.Utilities.IsValid(lastUsedPen);
                    if (__intnl_SystemBoolean_8)
                    {
                        lastUsedPen.SendCustomEvent("get_AllowCallPen");
                        __intnl_SystemBoolean_8 = lastUsedPen.GetProgramVariable("__0_get_AllowCallPen__ret");
                    }
                    __intnl_SystemBoolean_7 = __intnl_SystemBoolean_8;
                    if (__intnl_SystemBoolean_7)
                    {
                        lastUsedPen.SendCustomEvent("get_isHeld");
                        __intnl_SystemBoolean_7 = !lastUsedPen.GetProgramVariable("__0_get_isHeld__ret");
                    }
                    if (__intnl_SystemBoolean_7)
                    {
                        lastUsedPen.SendCustomEvent("_TakeOwnership");
                        __intnl_SystemObject_3 = lastUsedPen.GetProgramVariable("__0__TakeOwnership__ret");
                        __intnl_SystemBoolean_11 = __intnl_SystemObject_3;
                        __0_GetPen__ret = lastUsedPen;
                        return;
                    }
                    else
                    {
                        __intnl_SystemBoolean_11 = callablePenList.Count == 0;
                        if (__intnl_SystemBoolean_11)
                        {
                            __0_GetPen__ret = null;
                            return;
                        }
                        else
                        {
                            __lcl_indexList_SystemInt32Array_0 = new System.Int32[](callablePenList.Count);
                            __lcl_i_SystemInt32_0 = 0;
                            __lcl_n_SystemInt32_0 = callablePenList.Count;
                            __intnl_SystemBoolean_12 = __lcl_i_SystemInt32_0 < __lcl_n_SystemInt32_0;
                            while (__intnl_SystemBoolean_12)
                            {
                                __lcl_indexList_SystemInt32Array_0.Set(__lcl_i_SystemInt32_0, __lcl_i_SystemInt32_0);
                                __lcl_i_SystemInt32_0 = __lcl_i_SystemInt32_0 + 1;
                                __intnl_SystemBoolean_12 = __lcl_i_SystemInt32_0 < __lcl_n_SystemInt32_0;
                            }
                            VRC.SDKBase.Utilities.ShuffleArray(__lcl_indexList_SystemInt32Array_0);
                            __lcl_i_SystemInt32_0 = 0;
                            __lcl_n_SystemInt32_0 = __lcl_indexList_SystemInt32Array_0.Length;
                            __intnl_SystemBoolean_12 = __lcl_i_SystemInt32_0 < __lcl_n_SystemInt32_0;
                            while (__intnl_SystemBoolean_12)
                            {
                                __lcl_index_SystemInt32_0 = __lcl_indexList_SystemInt32Array_0.Get(__lcl_i_SystemInt32_0);
                                if (callablePenList.TryGetValue(__lcl_index_SystemInt32_0, null /* 15 */, out __lcl_penToken_VRCSDK3DataDataToken_0))
                                {
                                    __intnl_SystemObject_3 = __lcl_penToken_VRCSDK3DataDataToken_0.Reference;
                                    __lcl_pen_VRCUdonUdonBehaviour_1 = __intnl_SystemObject_3;
                                    __lcl_pen_VRCUdonUdonBehaviour_1.SendCustomEvent("get_isHeld");
                                    if (!__lcl_pen_VRCUdonUdonBehaviour_1.GetProgramVariable("__0_get_isHeld__ret"))
                                    {
                                        __lcl_pen_VRCUdonUdonBehaviour_1.SendCustomEvent("_TakeOwnership");
                                        lastUsedPen = __lcl_pen_VRCUdonUdonBehaviour_1;
                                        __0_GetPen__ret = __lcl_pen_VRCUdonUdonBehaviour_1;
                                        return;
                                    }
                                    else
                                    {
                                        goto label_bb_00000a9c;
                                    }
                                }
                                else
                                {
                                    goto label_bb_00000a9c;
                                }
                                __intnl_SystemBoolean_12 = __lcl_i_SystemInt32_0 < __lcl_n_SystemInt32_0;
                            }
                            __0_GetPen__ret = null;
                            return;
                        }
                    }
                }
            }
        label_bb_00000a9c:
            __lcl_i_SystemInt32_0 = __lcl_i_SystemInt32_0 + 1;
            goto label_bb_000008e4;
        label_bb_000008e4:
        }

        public void __0_HasInk()
        {
            VRC.SDK3.Data.DataToken __lcl_inkDictToken_VRCSDK3DataDataToken_0;
            if (inkDictMap.TryGetValue((VRC.SDK3.Data.DataToken)__1_penId__param, null /* 14 */, out __lcl_inkDictToken_VRCSDK3DataDataToken_0))
            {
                if (__lcl_inkDictToken_VRCSDK3DataDataToken_0.DataDictionary.ContainsKey((VRC.SDK3.Data.DataToken)__0_inkId__param))
                {
                    __0___0_HasInk__ret = true;
                    return;
                }
                else
                {
                    __0___0_HasInk__ret = false;
                    return;
                }
            }
            else
            {
                __0___0_HasInk__ret = false;
                return;
            }
        }

        public void __0_SetInk()
        {
            inkDictMap.get_Item((VRC.SDK3.Data.DataToken)__2_penId__param)
                .DataDictionary.set_Item((VRC.SDK3.Data.DataToken)__1_inkId__param, (VRC.SDK3.Data.DataToken)__0_inkInstance__param);
            return;
        }

        public void __0_RemoveInk()
        {
            UnityEngine.GameObject __lcl_ink_UnityEngineGameObject_0;
            VRC.SDK3.Data.DataToken __lcl_inkToken_VRCSDK3DataDataToken_0;
            VRC.SDK3.Data.DataDictionary __lcl_inkDict_VRCSDK3DataDataDictionary_0;
            VRC.SDK3.Data.DataToken __intnl_VRCSDK3DataDataToken_15;
            System.Boolean __intnl_SystemBoolean_20;
            __1_penId__param = __3_penId__param;
            __0_inkId__param = __2_inkId__param;
            __0_HasInk();
            if (__0___0_HasInk__ret)
            {
                __lcl_inkDict_VRCSDK3DataDataDictionary_0 = inkDictMap.get_Item((VRC.SDK3.Data.DataToken)__3_penId__param).DataDictionary;
                if (__lcl_inkDict_VRCSDK3DataDataDictionary_0.TryGetValue((VRC.SDK3.Data.DataToken)__2_inkId__param, null /* 15 */,
                                                                          out __lcl_inkToken_VRCSDK3DataDataToken_0))
                {
                    __lcl_ink_UnityEngineGameObject_0 = __lcl_inkToken_VRCSDK3DataDataToken_0.Reference;
                    if (VRC.SDKBase.Utilities.IsValid(__lcl_ink_UnityEngineGameObject_0))
                    {
                        UnityEngine.Object.Destroy(
                            __lcl_ink_UnityEngineGameObject_0.transform
                                .GetComponentInChildren(
                                    true,
                                    null /* "UnityEngine.MeshCollider, UnityEngine.PhysicsModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */)
                                .sharedMesh);
                        UnityEngine.Object.Destroy(__lcl_ink_UnityEngineGameObject_0);
                        __intnl_VRCSDK3DataDataToken_15 = (VRC.SDK3.Data.DataToken)__2_inkId__param;
                        __intnl_SystemBoolean_20 = __lcl_inkDict_VRCSDK3DataDataDictionary_0.Remove(__intnl_VRCSDK3DataDataToken_15);
                        __0___0_RemoveInk__ret = true;
                        return;
                    }
                    else
                    {
                        __intnl_VRCSDK3DataDataToken_15 = (VRC.SDK3.Data.DataToken)__2_inkId__param;
                        __intnl_SystemBoolean_20 = __lcl_inkDict_VRCSDK3DataDataDictionary_0.Remove(__intnl_VRCSDK3DataDataToken_15);
                        __0___0_RemoveInk__ret = false;
                        return;
                    }
                }
                else
                {
                    __0___0_RemoveInk__ret = false;
                    return;
                }
            }
            else
            {
                __0___0_RemoveInk__ret = false;
                return;
            }
        }

        public void __0_RemoveUserInk()
        {
            System.Boolean __intnl_SystemBoolean_23;
            UnityEngine.Vector3 __lcl__discard2_UnityEngineVector3_0;
            VRC.SDK3.Data.DataToken __lcl_inkIdToken_VRCSDK3DataDataToken_0;
            UnityEngine.GameObject __lcl_ink_UnityEngineGameObject_1;
            System.Int32 __lcl_inkOwnerId_SystemInt32_0;
            System.Boolean __intnl_SystemBoolean_21;
            VRC.SDK3.Data.DataToken __lcl_inkToken_VRCSDK3DataDataToken_1;
            System.Int32 __lcl_ownerId_SystemInt32_0;
            VRC.SDK3.Data.DataList __lcl_removedInkIdList_VRCSDK3DataDataList_0;
            VRC.SDK3.Data.DataDictionary __lcl_inkDict_VRCSDK3DataDataDictionary_1;
            System.Int32 __lcl_i_SystemInt32_1;
            System.Boolean __intnl_SystemBoolean_22;
            UnityEngine.Vector3 __lcl_inkOwnerIdVector_UnityEngineVector3_0;
            UnityEngine.Vector3 __lcl__discard1_UnityEngineVector3_0;
            System.Int32 __lcl_n_SystemInt32_1;
            VRC.SDK3.Data.DataList __lcl_inkIdList_VRCSDK3DataDataList_0;
            __lcl_inkDict_VRCSDK3DataDataDictionary_1 = inkDictMap.get_Item((VRC.SDK3.Data.DataToken)__4_penId__param).DataDictionary;
            __lcl_inkIdList_VRCSDK3DataDataList_0 = __lcl_inkDict_VRCSDK3DataDataDictionary_1.GetKeys();
            __lcl_removedInkIdList_VRCSDK3DataDataList_0 = new VRC.SDK3.Data.DataList();
            __1__intnlparam = __0_ownerIdVector__param;
            function_3();
            __lcl_ownerId_SystemInt32_0 = __0__intnlparam;
            __lcl_i_SystemInt32_1 = 0;
            __lcl_n_SystemInt32_1 = __lcl_inkIdList_VRCSDK3DataDataList_0.Count;
            __intnl_SystemBoolean_21 = __lcl_i_SystemInt32_1 < __lcl_n_SystemInt32_1;
            while (__intnl_SystemBoolean_21)
            {
                __intnl_SystemBoolean_22 =
                    __lcl_inkIdList_VRCSDK3DataDataList_0.TryGetValue(__lcl_i_SystemInt32_1, null /* 6 */, out __lcl_inkIdToken_VRCSDK3DataDataToken_0);
                if (__intnl_SystemBoolean_22)
                {
                    __intnl_SystemBoolean_23 = __lcl_inkDict_VRCSDK3DataDataDictionary_1.TryGetValue(__lcl_inkIdToken_VRCSDK3DataDataToken_0, null /* 15 */,
                                                                                                     out __lcl_inkToken_VRCSDK3DataDataToken_1);
                    if (__intnl_SystemBoolean_23)
                    {
                        __lcl_ink_UnityEngineGameObject_1 = __lcl_inkToken_VRCSDK3DataDataToken_1.Reference;
                        if (VRC.SDKBase.Utilities.IsValid(__lcl_ink_UnityEngineGameObject_1))
                        {
                            __3__intnlparam = __lcl_ink_UnityEngineGameObject_1;
                            __4__intnlparam = __lcl__discard1_UnityEngineVector3_0;
                            __5__intnlparam = __lcl__discard2_UnityEngineVector3_0;
                            __6__intnlparam = __lcl_inkOwnerIdVector_UnityEngineVector3_0;
                            function_4();
                            __lcl__discard1_UnityEngineVector3_0 = __4__intnlparam;
                            __lcl__discard2_UnityEngineVector3_0 = __5__intnlparam;
                            __lcl_inkOwnerIdVector_UnityEngineVector3_0 = __6__intnlparam;
                            if (__2__intnlparam)
                            {
                                __1__intnlparam = __lcl_inkOwnerIdVector_UnityEngineVector3_0;
                                function_3();
                                __lcl_inkOwnerId_SystemInt32_0 = __0__intnlparam;
                                if (!(__lcl_inkOwnerId_SystemInt32_0 != __lcl_ownerId_SystemInt32_0))
                                {
                                    UnityEngine.Object.Destroy(__lcl_ink_UnityEngineGameObject_1.transform.GetComponentInChildren(true, null /* "UnityEngine.MeshCollider, UnityEngine.PhysicsModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */).sharedMesh);
                                    UnityEngine.Object.Destroy(__lcl_ink_UnityEngineGameObject_1);
                                    __lcl_removedInkIdList_VRCSDK3DataDataList_0.Add(__lcl_inkIdToken_VRCSDK3DataDataToken_0);
                                }
                            }
                        }
                        else
                        {
                            __lcl_removedInkIdList_VRCSDK3DataDataList_0.Add(__lcl_inkIdToken_VRCSDK3DataDataToken_0);
                        }
                    }
                }
                __lcl_i_SystemInt32_1 = __lcl_i_SystemInt32_1 + 1;
                __intnl_SystemBoolean_21 = __lcl_i_SystemInt32_1 < __lcl_n_SystemInt32_1;
            }
            __lcl_i_SystemInt32_1 = 0;
            __lcl_n_SystemInt32_1 = __lcl_removedInkIdList_VRCSDK3DataDataList_0.Count;
            __intnl_SystemBoolean_21 = __lcl_i_SystemInt32_1 < __lcl_n_SystemInt32_1;
            while (__intnl_SystemBoolean_21)
            {
                __intnl_SystemBoolean_22 =
                    __lcl_removedInkIdList_VRCSDK3DataDataList_0.TryGetValue(__lcl_i_SystemInt32_1, null /* 6 */, out __lcl_inkIdToken_VRCSDK3DataDataToken_0);
                if (__intnl_SystemBoolean_22)
                {
                    __intnl_SystemBoolean_23 = __lcl_inkDict_VRCSDK3DataDataDictionary_1.Remove(__lcl_inkIdToken_VRCSDK3DataDataToken_0);
                }
                __lcl_i_SystemInt32_1 = __lcl_i_SystemInt32_1 + 1;
                __intnl_SystemBoolean_21 = __lcl_i_SystemInt32_1 < __lcl_n_SystemInt32_1;
            }
            __0___0_RemoveUserInk__ret = true;
            return;
        }

        public void __0_Clear()
        {
            UnityEngine.GameObject __lcl_ink_UnityEngineGameObject_2;
            VRC.SDK3.Data.DataToken __lcl_inkToken_VRCSDK3DataDataToken_2;
            VRC.SDK3.Data.DataList __lcl_inkTokens_VRCSDK3DataDataList_0;
            VRC.SDK3.Data.DataDictionary __lcl_inkDict_VRCSDK3DataDataDictionary_2;
            System.Int32 __lcl_i_SystemInt32_2;
            System.Int32 __lcl_n_SystemInt32_2;
            VRC.SDK3.Data.DataToken __lcl_inkDictToken_VRCSDK3DataDataToken_1;
            if (inkDictMap.TryGetValue((VRC.SDK3.Data.DataToken)__5_penId__param, null /* 14 */, out __lcl_inkDictToken_VRCSDK3DataDataToken_1))
            {
                __lcl_inkDict_VRCSDK3DataDataDictionary_2 = __lcl_inkDictToken_VRCSDK3DataDataToken_1.DataDictionary;
                __lcl_inkTokens_VRCSDK3DataDataList_0 = __lcl_inkDict_VRCSDK3DataDataDictionary_2.GetValues();
                __lcl_i_SystemInt32_2 = 0;
                __lcl_n_SystemInt32_2 = __lcl_inkTokens_VRCSDK3DataDataList_0.Count;
                while (__lcl_i_SystemInt32_2 < __lcl_n_SystemInt32_2)
                {
                    if (__lcl_inkTokens_VRCSDK3DataDataList_0.TryGetValue(__lcl_i_SystemInt32_2, null /* 15 */, out __lcl_inkToken_VRCSDK3DataDataToken_2))
                    {
                        __lcl_ink_UnityEngineGameObject_2 = __lcl_inkToken_VRCSDK3DataDataToken_2.Reference;
                        if (VRC.SDKBase.Utilities.IsValid(__lcl_ink_UnityEngineGameObject_2))
                        {
                            UnityEngine.Object.Destroy(
                                __lcl_ink_UnityEngineGameObject_2.transform
                                    .GetComponentInChildren(
                                        true,
                                        null /* "UnityEngine.MeshCollider, UnityEngine.PhysicsModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */)
                                    .sharedMesh);
                            UnityEngine.Object.Destroy(__lcl_ink_UnityEngineGameObject_2);
                        }
                    }
                    __lcl_i_SystemInt32_2 = __lcl_i_SystemInt32_2 + 1;
                }
                __lcl_inkDict_VRCSDK3DataDataDictionary_2.Clear();
                return;
            }
            else
            {
                return;
            }
        }

        void function_2()
        {
            __8__intnlparam = __0_c__param;
            function_5();
            __0___0_ColorBeginTag__ret = System.String.Format("<color=\"#{0}\">", __7__intnlparam);
            return;
        }

        void get_logPrefix()
        {
            if (!System.String.IsNullOrEmpty(_logPrefix))
            {
                __0_get_logPrefix__ret = _logPrefix;
            }
            else
            {
                __0_c__param = logColor;
                function_2();
                __gintnl_SystemObjectArray_0.Set(0, __0___0_ColorBeginTag__ret);
                __gintnl_SystemObjectArray_0.Set(1, "QvPen");
                __gintnl_SystemObjectArray_0.Set(2, "Udon");
                __gintnl_SystemObjectArray_0.Set(3, "QvPen_Manager");
                __gintnl_SystemObjectArray_0.Set(4, "</color>");
                _logPrefix = System.String.Format("[{0}{1}.{2}.{3}{4}] ", __gintnl_SystemObjectArray_0);
                __0_get_logPrefix__ret = _logPrefix;
            }
            return;
        }

        void function_3()
        {
            __1__intnlparam = __1__intnlparam * 4.0f;
            __0__intnlparam = UnityEngine.Mathf.RoundToInt(__1__intnlparam.x) + UnityEngine.Mathf.RoundToInt(__1__intnlparam.y) * 360 +
                              UnityEngine.Mathf.RoundToInt(__1__intnlparam.z) * 129600;
            return;
        }

        void function_4()
        {
            UnityEngine.Transform __lcl_idHolder_UnityEngineTransform_0;
            if (VRC.SDKBase.Utilities.IsValid(__3__intnlparam))
            {
                if (__3__intnlparam.transform.childCount < 2)
                {
                    __4__intnlparam = __const_UnityEngineVector3_1;
                    __5__intnlparam = __const_UnityEngineVector3_1;
                    __6__intnlparam = __const_UnityEngineVector3_1;
                    __2__intnlparam = false;
                    return;
                }
                else
                {
                    __lcl_idHolder_UnityEngineTransform_0 = __3__intnlparam.transform.GetChild(1);
                    if (VRC.SDKBase.Utilities.IsValid(__lcl_idHolder_UnityEngineTransform_0))
                    {
                        __4__intnlparam = __lcl_idHolder_UnityEngineTransform_0.localPosition;
                        __5__intnlparam = __lcl_idHolder_UnityEngineTransform_0.localScale;
                        __6__intnlparam = __lcl_idHolder_UnityEngineTransform_0.localEulerAngles;
                        __2__intnlparam = true;
                        return;
                    }
                    else
                    {
                        __4__intnlparam = __const_UnityEngineVector3_1;
                        __5__intnlparam = __const_UnityEngineVector3_1;
                        __6__intnlparam = __const_UnityEngineVector3_1;
                        __2__intnlparam = false;
                        return;
                    }
                }
            }
            else
            {
                __4__intnlparam = __const_UnityEngineVector3_1;
                __5__intnlparam = __const_UnityEngineVector3_1;
                __6__intnlparam = __const_UnityEngineVector3_1;
                __2__intnlparam = false;
                return;
            }
        }

        void function_5()
        {
            __8__intnlparam = __8__intnlparam * 255.0f;
            __7__intnlparam = System.String.Format("{0:x2}{1:x2}{2:x2}", UnityEngine.Mathf.RoundToInt(__8__intnlparam.r),
                                                   UnityEngine.Mathf.RoundToInt(__8__intnlparam.g), UnityEngine.Mathf.RoundToInt(__8__intnlparam.b));
            return;
        }
    }
}