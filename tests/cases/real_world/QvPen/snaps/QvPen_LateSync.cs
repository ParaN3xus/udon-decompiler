// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.UdonScript
{
    public class QvPen_LateSync : UdonSharpBehaviour
    {
        VRC.Udon.UdonBehaviour _pen_k__BackingField = null;
        UnityEngine.Transform inkPoolSynced = null;
        UnityEngine.Transform inkPoolNotSynced = null;
        UnityEngine.LineRenderer[] linesBuffer = null /* [] */;
        System.Int32 inkIndex = -1;
        System.Boolean forceStart = false;
        UnityEngine.Vector3[] _syncedData = null;
        System.Boolean _isNetworkSettled = false;
        System.Boolean isInUseSyncBuffer = false;
        System.Int32 retryCount = 0;
        UnityEngine.LineRenderer nextInk = null;
        UnityEngine.Vector3 beginSignal = null /* "(271828200.00, 1.00, 62831.85)" */;
        UnityEngine.Vector3 endSignal = null /* "(271828200.00, 0.00, 62831.85)" */;
        UnityEngine.Vector3 errorSignal = null /* "(271828200.00, -1.00, 62831.85)" */;
        VRC.Udon.UdonBehaviour __0_get_pen__ret = null;
        VRC.Udon.UdonBehaviour __0_value__param = null;
        UnityEngine.Transform __0_get_InkPoolSynced__ret = null;
        UnityEngine.Transform __0_get_InkPoolNotSynced__ret = null;
        VRC.Udon.UdonBehaviour __0_pen__param = null;
        UnityEngine.Vector3[] __0_get_syncedData__ret = null;
        UnityEngine.Vector3[] __1_value__param = null;
        UnityEngine.Vector3[] __1_data__param = null;
        System.Boolean __0_get_isNetworkSettled__ret = false;
        UnityEngine.Vector3[] __0_data__param = null;
        VRC.Udon.Common.SerializationResult onPostSerializationResult = null /* {"success": false, "byteCount": 0} */;
        UnityEngine.Vector3 __0___0_GetCalibrationSignal__ret = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector3[] __2_data__param = null;
        UnityEngine.LineRenderer __0_GetNextInk__ret = null;
        System.Boolean __0__intnlparam = false;
        UnityEngine.GameObject __1__intnlparam = null;
        UnityEngine.Vector3 __2__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector3 __3__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector3 __4__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector3 __0___0_GetPenIdVector__ret = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector3[] __4_data__param = null;
        UnityEngine.Vector3 __0___0_GetData__ret = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector3[] __3_data__param = null;
        System.Int32 __0_index__param = 0;
        UnityEngine.Vector3 __const_UnityEngineVector3_0 = null /* "(0.00, 0.00, 0.00)" */;

        public void get_pen()
        {
            __0_get_pen__ret = _pen_k__BackingField;
            return;
        }

        void function_0()
        {
            _pen_k__BackingField = __0_value__param;
            return;
        }

        public void get_InkPoolSynced()
        {
            __0_get_InkPoolSynced__ret = inkPoolSynced;
            return;
        }

        public void get_InkPoolNotSynced()
        {
            __0_get_InkPoolNotSynced__ret = inkPoolNotSynced;
            return;
        }

        public void __0__RegisterPen()
        {
            __0_value__param = __0_pen__param;
            function_0();
            return;
        }

        public void _onPlayerJoined()
        {
            System.Boolean __intnl_SystemBoolean_0 = false;

            __intnl_SystemBoolean_0 = VRC.SDKBase.VRCPlayerApi.GetPlayerCount() > 1;
            if (__intnl_SystemBoolean_0)
            {
                __intnl_SystemBoolean_0 = VRC.SDKBase.Networking.IsOwner(this.gameObject);
            }
            if (__intnl_SystemBoolean_0)
            {
                StartSync();
            }
            return;
        }

        public void _onOwnershipTransferred()
        {
            System.Boolean __intnl_SystemBoolean_1 = false;

            __intnl_SystemBoolean_1 = VRC.SDKBase.VRCPlayerApi.GetPlayerCount() > 1;
            if (__intnl_SystemBoolean_1)
            {
                __intnl_SystemBoolean_1 = VRC.SDKBase.Networking.IsOwner(this.gameObject);
            }
            if (__intnl_SystemBoolean_1)
            {
                this.SendCustomEventDelayedSeconds("StartSync", 1.84f * (1.0f + UnityEngine.Random.value), null /* 0 */);
            }
            return;
        }

        public void StartSync()
        {
            forceStart = true;
            retryCount = 0;
            function_4();
            return;
        }

        void get_syncedData()
        {
            __0_get_syncedData__ret = _syncedData;
            return;
        }

        void function_1()
        {
            UnityEngine.Vector3[] __intnl_UnityEngineVector3Array_0 = null;
            VRC.Udon.UdonBehaviour __intnl_VRCUdonUdonBehaviour_0 = null;

            if (forceStart)
            {
                _syncedData = new UnityEngine.Vector3[](2);
                __intnl_UnityEngineVector3Array_0 = _syncedData;
                get_pen();
                __intnl_VRCUdonUdonBehaviour_0 = __0_get_pen__ret;
                __0_get_pen__ret.SendCustomEvent("get_penIdVector");
                __intnl_UnityEngineVector3Array_0.Set(0, __0_get_pen__ret.GetProgramVariable("__0_get_penIdVector__ret"));
                _syncedData.Set(1, beginSignal);
                if (VRC.SDKBase.Networking.IsOwner(this.gameObject))
                {
                    _RequestSendPackage();
                }
            }
            else
            {
                _syncedData = __1_value__param;
                if (VRC.SDKBase.Networking.IsOwner(this.gameObject))
                {
                    _RequestSendPackage();
                }
                else
                {
                    __1_data__param = _syncedData;
                    function_3();
                }
            }
            return;
        }

        void get_isNetworkSettled()
        {
            System.Boolean __intnl_SystemBoolean_3 = false;

            __intnl_SystemBoolean_3 = _isNetworkSettled;
            if (!__intnl_SystemBoolean_3)
            {
                _isNetworkSettled = VRC.SDKBase.Networking.IsNetworkSettled;
                __intnl_SystemBoolean_3 = _isNetworkSettled;
            }
            __0_get_isNetworkSettled__ret = __intnl_SystemBoolean_3;
            return;
        }

        public void _RequestSendPackage()
        {
            System.Boolean __intnl_SystemBoolean_4 = false;

            __intnl_SystemBoolean_4 = VRC.SDKBase.VRCPlayerApi.GetPlayerCount() > 1;
            if (__intnl_SystemBoolean_4)
            {
                __intnl_SystemBoolean_4 = VRC.SDKBase.Networking.IsOwner(this.gameObject);
            }
            if (__intnl_SystemBoolean_4)
            {
                get_isNetworkSettled();
                if (!__0_get_isNetworkSettled__ret)
                {
                    this.SendCustomEventDelayedSeconds("_RequestSendPackage", 1.84f, null /* 0 */);
                }
                else
                {
                    isInUseSyncBuffer = true;
                    this.RequestSerialization();
                }
            }
            return;
        }

        void function_2()
        {
            if (!isInUseSyncBuffer)
            {
                __1_value__param = __0_data__param;
                function_1();
            }
            return;
        }

        public void _onPreSerialization()
        {
            get_syncedData();
            _syncedData = __0_get_syncedData__ret;
            return;
        }

        public void _onDeserialization()
        {
            __1_value__param = _syncedData;
            function_1();
            return;
        }

        public void _onPostSerialization()
        {
            UnityEngine.Vector3 __lcl_signal_UnityEngineVector3_0 = null /* "(0.00, 0.00, 0.00)" */;
            UnityEngine.LineRenderer __lcl_ink_UnityEngineLineRenderer_0 = null;
            System.Int32 __lcl_totalLength_SystemInt32_0 = 0;
            VRC.SDK3.Data.DataList __lcl_dataList_VRCSDK3DataDataList_0 = null;
            VRC.SDK3.Data.DataList __lcl_lengthList_VRCSDK3DataDataList_0 = null;
            UnityEngine.Vector3[] __lcl_lengthVectors_UnityEngineVector3Array_0 = null;
            System.Int32 __intnl_SystemInt32_3 = 0;
            UnityEngine.Vector3[] __lcl_joinedData_UnityEngineVector3Array_0 = null;
            System.Int32 __lcl_index_SystemInt32_0 = 0;
            VRC.Udon.UdonBehaviour __intnl_VRCUdonUdonBehaviour_1 = null;
            UnityEngine.Vector3 __intnl_UnityEngineVector3_1 = null /* "(0.00, 0.00, 0.00)" */;
            UnityEngine.Vector3 __lcl__discard_UnityEngineVector3_0 = null /* "(0.00, 0.00, 0.00)" */;
            UnityEngine.Vector3 __lcl_inkIdVector_UnityEngineVector3_0 = null /* "(0.00, 0.00, 0.00)" */;
            UnityEngine.Vector3 __lcl_ownerIdVector_UnityEngineVector3_0 = null /* "(0.00, 0.00, 0.00)" */;
            UnityEngine.Vector3[] __lcl_data_UnityEngineVector3Array_0 = null;
            System.Int32 __lcl_length_SystemInt32_0 = 0;
            System.Int32 __lcl_i_SystemInt32_0 = 0;
            System.Int32 __lcl_n_SystemInt32_0 = 0;
            VRC.SDK3.Data.DataToken __lcl_lengthToken_VRCSDK3DataDataToken_0 = null /* "Null" */;
            VRC.SDK3.Data.DataToken __lcl_dataToken_VRCSDK3DataDataToken_0 = null /* "Null" */;

            isInUseSyncBuffer = false;
            if (onPostSerializationResult.success)
            {
                retryCount = 0;
                get_syncedData();
                __2_data__param = __0_get_syncedData__ret;
                GetCalibrationSignal();
                __lcl_signal_UnityEngineVector3_0 = __0___0_GetCalibrationSignal__ret;
                if (!(__lcl_signal_UnityEngineVector3_0 == errorSignal))
                {
                    if (__lcl_signal_UnityEngineVector3_0 == beginSignal)
                    {
                        forceStart = false;
                        linesBuffer = inkPoolSynced.transform.GetComponentsInChildren(
                            null /* "UnityEngine.LineRenderer, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
                        inkIndex = -1;
                        nextInk = null;
                    }
                    else
                    {
                        if (__lcl_signal_UnityEngineVector3_0 == endSignal)
                        {
                            linesBuffer = new UnityEngine.LineRenderer[](0);
                            __1_value__param = new UnityEngine.Vector3[](0);
                            function_1();
                            isInUseSyncBuffer = false;
                            return;
                        }
                    }
                    __lcl_ink_UnityEngineLineRenderer_0 = nextInk;
                    if (!VRC.SDKBase.Utilities.IsValid(__lcl_ink_UnityEngineLineRenderer_0))
                    {
                        function_6();
                        __lcl_ink_UnityEngineLineRenderer_0 = __0_GetNextInk__ret;
                    }
                    if (VRC.SDKBase.Utilities.IsValid(__lcl_ink_UnityEngineLineRenderer_0))
                    {
                        __lcl_totalLength_SystemInt32_0 = 0;
                        __lcl_dataList_VRCSDK3DataDataList_0 = new VRC.SDK3.Data.DataList();
                        __lcl_lengthList_VRCSDK3DataDataList_0 = new VRC.SDK3.Data.DataList();
                        while (VRC.SDKBase.Utilities.IsValid(__lcl_ink_UnityEngineLineRenderer_0))
                        {
                            __1__intnlparam = __lcl_ink_UnityEngineLineRenderer_0.gameObject;
                            __2__intnlparam = __lcl__discard_UnityEngineVector3_0;
                            __3__intnlparam = __lcl_inkIdVector_UnityEngineVector3_0;
                            __4__intnlparam = __lcl_ownerIdVector_UnityEngineVector3_0;
                            function_7();
                            __lcl__discard_UnityEngineVector3_0 = __2__intnlparam;
                            __lcl_inkIdVector_UnityEngineVector3_0 = __3__intnlparam;
                            __lcl_ownerIdVector_UnityEngineVector3_0 = __4__intnlparam;
                            if (!__0__intnlparam)
                            {
                                function_6();
                                __lcl_ink_UnityEngineLineRenderer_0 = __0_GetNextInk__ret;
                                continue;
                            }
                            get_pen();
                            __0_get_pen__ret.SetProgramVariable("__0_lineRenderer__param", __lcl_ink_UnityEngineLineRenderer_0);
                            __0_get_pen__ret.SetProgramVariable("__2_mode__param", 2);
                            __0_get_pen__ret.SetProgramVariable("__1_inkIdVector__param", __lcl_inkIdVector_UnityEngineVector3_0);
                            __0_get_pen__ret.SetProgramVariable("__1_ownerIdVector__param", __lcl_ownerIdVector_UnityEngineVector3_0);
                            __0_get_pen__ret.SendCustomEvent("__0__PackData");
                            __lcl_data_UnityEngineVector3Array_0 = __0_get_pen__ret.GetProgramVariable("__0___0__PackData__ret");
                            __lcl_length_SystemInt32_0 = __lcl_data_UnityEngineVector3Array_0.Length;
                            __lcl_dataList_VRCSDK3DataDataList_0.Add(new VRC.SDK3.Data.DataToken(__lcl_data_UnityEngineVector3Array_0));
                            __lcl_lengthList_VRCSDK3DataDataList_0.Add(__lcl_length_SystemInt32_0);
                            __lcl_totalLength_SystemInt32_0 = __lcl_totalLength_SystemInt32_0 + __lcl_length_SystemInt32_0;
                            function_6();
                            __lcl_ink_UnityEngineLineRenderer_0 = __0_GetNextInk__ret;
                            if (!VRC.SDKBase.Utilities.IsValid(__lcl_ink_UnityEngineLineRenderer_0))
                            {
                                nextInk = null;
                            }
                            else
                            {
                                if (!(__lcl_totalLength_SystemInt32_0 + __lcl_ink_UnityEngineLineRenderer_0.positionCount > 80))
                                {
                                    continue;
                                }
                                nextInk = __lcl_ink_UnityEngineLineRenderer_0;
                            }
                            break;
                        }
                        __lcl_lengthVectors_UnityEngineVector3Array_0 = new UnityEngine.Vector3[]((__lcl_lengthList_VRCSDK3DataDataList_0.Count + 2) / 3);
                        __lcl_i_SystemInt32_0 = 0;
                        __lcl_n_SystemInt32_0 = __lcl_lengthList_VRCSDK3DataDataList_0.Count;
                        while (__lcl_i_SystemInt32_0 < __lcl_n_SystemInt32_0)
                        {
                            if (__lcl_lengthList_VRCSDK3DataDataList_0.TryGetValue(__lcl_i_SystemInt32_0, null /* 6 */,
                                                                                   out __lcl_lengthToken_VRCSDK3DataDataToken_0))
                            {
                                __intnl_UnityEngineVector3_1 = __lcl_lengthVectors_UnityEngineVector3Array_0.Get(__lcl_i_SystemInt32_0 / 3);
                                __intnl_UnityEngineVector3_1.set_Item(__lcl_i_SystemInt32_0 % 3,
                                                                      System.Convert.ToSingle(__lcl_lengthToken_VRCSDK3DataDataToken_0.Int));
                                __lcl_lengthVectors_UnityEngineVector3Array_0.Set(__lcl_i_SystemInt32_0 / 3, __intnl_UnityEngineVector3_1);
                            }
                            __lcl_i_SystemInt32_0 = __lcl_i_SystemInt32_0 + 1;
                        }
                        __lcl_joinedData_UnityEngineVector3Array_0 =
                            new UnityEngine.Vector3[](2 + __lcl_lengthVectors_UnityEngineVector3Array_0.Length + __lcl_totalLength_SystemInt32_0);
                        __lcl_index_SystemInt32_0 = 0;
                        get_pen();
                        __intnl_VRCUdonUdonBehaviour_1 = __0_get_pen__ret;
                        __0_get_pen__ret.SendCustomEvent("get_penIdVector");
                        __lcl_joinedData_UnityEngineVector3Array_0.Set(0, __0_get_pen__ret.GetProgramVariable("__0_get_penIdVector__ret"));
                        __lcl_index_SystemInt32_0 = __lcl_index_SystemInt32_0 + 1;
                        __lcl_joinedData_UnityEngineVector3Array_0.Set(
                            1, new UnityEngine.Vector3(System.Convert.ToSingle(__lcl_lengthList_VRCSDK3DataDataList_0.Count),
                                                       System.Convert.ToSingle(__lcl_joinedData_UnityEngineVector3Array_0.Length), 0.0f));
                        __lcl_index_SystemInt32_0 = __lcl_index_SystemInt32_0 + 1;
                        System.Array.Copy(__lcl_lengthVectors_UnityEngineVector3Array_0, 0, __lcl_joinedData_UnityEngineVector3Array_0,
                                          __lcl_index_SystemInt32_0, __lcl_lengthVectors_UnityEngineVector3Array_0.Length);
                        __lcl_index_SystemInt32_0 = __lcl_index_SystemInt32_0 + __lcl_lengthVectors_UnityEngineVector3Array_0.Length;
                        __lcl_i_SystemInt32_0 = 0;
                        __lcl_n_SystemInt32_0 = __lcl_dataList_VRCSDK3DataDataList_0.Count;
                        while (__lcl_i_SystemInt32_0 < __lcl_n_SystemInt32_0)
                        {
                            if (__lcl_dataList_VRCSDK3DataDataList_0.TryGetValue(__lcl_i_SystemInt32_0, null /* 15 */,
                                                                                 out __lcl_dataToken_VRCSDK3DataDataToken_0))
                            {
                                __lcl_data_UnityEngineVector3Array_0 = __lcl_dataToken_VRCSDK3DataDataToken_0.Reference;
                                System.Array.Copy(__lcl_data_UnityEngineVector3Array_0, 0, __lcl_joinedData_UnityEngineVector3Array_0,
                                                  __lcl_index_SystemInt32_0, __lcl_data_UnityEngineVector3Array_0.Length);
                                __lcl_index_SystemInt32_0 = __lcl_index_SystemInt32_0 + __lcl_data_UnityEngineVector3Array_0.Length;
                            }
                            __lcl_i_SystemInt32_0 = __lcl_i_SystemInt32_0 + 1;
                        }
                        __lcl_dataList_VRCSDK3DataDataList_0.Clear();
                        __lcl_lengthList_VRCSDK3DataDataList_0.Clear();
                        __0_data__param = __lcl_joinedData_UnityEngineVector3Array_0;
                        function_2();
                    }
                    else
                    {
                        function_5();
                    }
                }
            }
            else
            {
                __intnl_SystemInt32_3 = retryCount;
                retryCount = __intnl_SystemInt32_3 + 1;
                if (__intnl_SystemInt32_3 < 3)
                {
                    this.SendCustomEventDelayedSeconds("_RequestSendPackage", 1.84f, null /* 0 */);
                }
            }
            return;
        }

        void function_3()
        {
            System.Boolean __intnl_SystemBoolean_14 = false;
            UnityEngine.Vector3 __lcl_penIdVector_UnityEngineVector3_0 = null /* "(0.00, 0.00, 0.00)" */;
            System.Boolean __intnl_SystemBoolean_15 = false;
            System.Int32 __lcl_currentSyncState_SystemInt32_0 = 0;
            UnityEngine.Vector3 __lcl_signal_UnityEngineVector3_1 = null /* "(0.00, 0.00, 0.00)" */;
            System.Int32 __lcl_index_SystemInt32_1 = 0;
            System.Int32 __lcl_length_SystemInt32_1 = 0;
            System.Int32 __lcl_check_SystemInt32_0 = 0;
            UnityEngine.Vector3[] __lcl_lengthVectors_UnityEngineVector3Array_1 = null;
            System.Int32 __lcl_i_SystemInt32_1 = 0;
            System.Int32 __lcl_dataLength_SystemInt32_0 = 0;
            UnityEngine.Vector3[] __lcl_stroke_UnityEngineVector3Array_0 = null;

            __intnl_SystemBoolean_14 = _syncedData == null;
            if (!__intnl_SystemBoolean_14)
            {
                __intnl_SystemBoolean_14 = _syncedData.Length < 2;
            }
            if (!__intnl_SystemBoolean_14)
            {
                __4_data__param = __1_data__param;
                GetPenIdVector();
                __lcl_penIdVector_UnityEngineVector3_0 = __0___0_GetPenIdVector__ret;
                get_pen();
                __intnl_SystemBoolean_15 = VRC.SDKBase.Utilities.IsValid(__0_get_pen__ret);
                if (__intnl_SystemBoolean_15)
                {
                    get_pen();
                    __0_get_pen__ret.SetProgramVariable("__0_idVector__param", __lcl_penIdVector_UnityEngineVector3_0);
                    __0_get_pen__ret.SendCustomEvent("__0__CheckId");
                    __intnl_SystemBoolean_15 = __0_get_pen__ret.GetProgramVariable("__0___0__CheckId__ret");
                }
                if (__intnl_SystemBoolean_15)
                {
                    get_pen();
                    __lcl_currentSyncState_SystemInt32_0 = System.Convert.ToInt32(__0_get_pen__ret.GetProgramVariable("currentSyncState"));
                    if (!(__lcl_currentSyncState_SystemInt32_0 == 2))
                    {
                        __2_data__param = __1_data__param;
                        GetCalibrationSignal();
                        __lcl_signal_UnityEngineVector3_1 = __0___0_GetCalibrationSignal__ret;
                        if (__lcl_signal_UnityEngineVector3_1 == beginSignal)
                        {
                            if (__lcl_currentSyncState_SystemInt32_0 == 0)
                            {
                                get_pen();
                                __0_get_pen__ret.SetProgramVariable("currentSyncState", 1);
                            }
                        }
                        else
                        {
                            if (__lcl_signal_UnityEngineVector3_1 == endSignal)
                            {
                                if (__lcl_currentSyncState_SystemInt32_0 == 1)
                                {
                                    get_pen();
                                    __0_get_pen__ret.SetProgramVariable("currentSyncState", 2);
                                }
                            }
                            else
                            {
                                if (__1_data__param.Length > 2)
                                {
                                    __lcl_index_SystemInt32_1 = 1;
                                    __lcl_length_SystemInt32_1 =
                                        System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__1_data__param.Get(__lcl_index_SystemInt32_1).x)));
                                    __lcl_check_SystemInt32_0 =
                                        System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__1_data__param.Get(__lcl_index_SystemInt32_1).y)));
                                    __lcl_index_SystemInt32_1 = __lcl_index_SystemInt32_1 + 1;
                                    if (!(__lcl_check_SystemInt32_0 != __1_data__param.Length))
                                    {
                                        __lcl_lengthVectors_UnityEngineVector3Array_1 = new UnityEngine.Vector3[]((__lcl_length_SystemInt32_1 + 2) / 3);
                                        System.Array.Copy(__1_data__param, __lcl_index_SystemInt32_1, __lcl_lengthVectors_UnityEngineVector3Array_1, 0,
                                                          __lcl_lengthVectors_UnityEngineVector3Array_1.Length);
                                        __lcl_index_SystemInt32_1 = __lcl_index_SystemInt32_1 + __lcl_lengthVectors_UnityEngineVector3Array_1.Length;
                                        __lcl_i_SystemInt32_1 = 0;
                                        while (__lcl_i_SystemInt32_1 < __lcl_length_SystemInt32_1)
                                        {
                                            __lcl_dataLength_SystemInt32_0 = System.Convert.ToInt32(System.Math.Truncate(
                                                System.Convert.ToDouble(__lcl_lengthVectors_UnityEngineVector3Array_1.Get(__lcl_i_SystemInt32_1 / 3)
                                                                            .get_Item(__lcl_i_SystemInt32_1 % 3))));
                                            __lcl_stroke_UnityEngineVector3Array_0 = new UnityEngine.Vector3[](__lcl_dataLength_SystemInt32_0);
                                            System.Array.Copy(__1_data__param, __lcl_index_SystemInt32_1, __lcl_stroke_UnityEngineVector3Array_0, 0,
                                                              __lcl_dataLength_SystemInt32_0);
                                            __lcl_index_SystemInt32_1 = __lcl_index_SystemInt32_1 + __lcl_dataLength_SystemInt32_0;
                                            get_pen();
                                            __0_get_pen__ret.SetProgramVariable("__5_data__param", __lcl_stroke_UnityEngineVector3Array_0);
                                            __0_get_pen__ret.SetProgramVariable("__0_targetMode__param", 1);
                                            __0_get_pen__ret.SendCustomEvent("__0__UnpackData");
                                            __lcl_i_SystemInt32_1 = __lcl_i_SystemInt32_1 + 1;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            return;
        }

        void function_4()
        {
            UnityEngine.Vector3[] __intnl_UnityEngineVector3Array_2 = null;
            VRC.Udon.UdonBehaviour __intnl_VRCUdonUdonBehaviour_2 = null;

            __intnl_UnityEngineVector3Array_2 = new UnityEngine.Vector3[](2);
            get_pen();
            __intnl_VRCUdonUdonBehaviour_2 = __0_get_pen__ret;
            __0_get_pen__ret.SendCustomEvent("get_penIdVector");
            __intnl_UnityEngineVector3Array_2.Set(0, __0_get_pen__ret.GetProgramVariable("__0_get_penIdVector__ret"));
            __intnl_UnityEngineVector3Array_2.Set(1, beginSignal);
            __0_data__param = __intnl_UnityEngineVector3Array_2;
            function_2();
            return;
        }

        void function_5()
        {
            UnityEngine.Vector3[] __intnl_UnityEngineVector3Array_3 = null;
            VRC.Udon.UdonBehaviour __intnl_VRCUdonUdonBehaviour_3 = null;

            __intnl_UnityEngineVector3Array_3 = new UnityEngine.Vector3[](2);
            get_pen();
            __intnl_VRCUdonUdonBehaviour_3 = __0_get_pen__ret;
            __0_get_pen__ret.SendCustomEvent("get_penIdVector");
            __intnl_UnityEngineVector3Array_3.Set(0, __0_get_pen__ret.GetProgramVariable("__0_get_penIdVector__ret"));
            __intnl_UnityEngineVector3Array_3.Set(1, endSignal);
            __0_data__param = __intnl_UnityEngineVector3Array_3;
            function_2();
            return;
        }

        void GetCalibrationSignal()
        {
            System.Boolean __intnl_SystemBoolean_23 = false;

            __intnl_SystemBoolean_23 = __2_data__param != null;
            if (__intnl_SystemBoolean_23)
            {
                __intnl_SystemBoolean_23 = __2_data__param.Length > 1;
            }
            if (__intnl_SystemBoolean_23)
            {
                __0___0_GetCalibrationSignal__ret = __2_data__param.Get(1);
            }
            else
            {
                __0___0_GetCalibrationSignal__ret = errorSignal;
            }
            return;
        }

        void GetData()
        {
            System.Boolean __intnl_SystemBoolean_24 = false;

            __intnl_SystemBoolean_24 = __3_data__param != null;
            if (__intnl_SystemBoolean_24)
            {
                __intnl_SystemBoolean_24 = __3_data__param.Length > __0_index__param;
            }
            if (__intnl_SystemBoolean_24)
            {
                __0___0_GetData__ret = __3_data__param.Get(__3_data__param.Length - 1 - __0_index__param);
            }
            else
            {
                __0___0_GetData__ret = errorSignal;
            }
            return;
        }

        void GetPenIdVector()
        {
            System.Boolean __intnl_SystemBoolean_25 = false;

            __intnl_SystemBoolean_25 = __4_data__param != null;
            if (__intnl_SystemBoolean_25)
            {
                __intnl_SystemBoolean_25 = __4_data__param.Length > 1;
            }
            if (__intnl_SystemBoolean_25)
            {
                __3_data__param = __4_data__param;
                __0_index__param = 1;
                GetData();
                __0___0_GetPenIdVector__ret = __0___0_GetData__ret;
            }
            else
            {
                __0___0_GetPenIdVector__ret = errorSignal;
            }
            return;
        }

        void function_6()
        {
            UnityEngine.LineRenderer __lcl_ink_UnityEngineLineRenderer_1 = null;

            inkIndex = UnityEngine.Mathf.Max(-1, inkIndex);
            while (true)
            {
                inkIndex = inkIndex + 1;
                if (inkIndex < linesBuffer.Length)
                {
                    __lcl_ink_UnityEngineLineRenderer_1 = linesBuffer.Get(inkIndex);
                    if (!VRC.SDKBase.Utilities.IsValid(__lcl_ink_UnityEngineLineRenderer_1))
                    {
                        continue;
                    }
                    __0_GetNextInk__ret = __lcl_ink_UnityEngineLineRenderer_1;
                    return;
                }
                break;
            }
            __0_GetNextInk__ret = null;
            return;
        }

        void function_7()
        {
            UnityEngine.Transform __lcl_idHolder_UnityEngineTransform_0 = null;

            if (!VRC.SDKBase.Utilities.IsValid(__1__intnlparam))
            {
                __2__intnlparam = __const_UnityEngineVector3_0;
                __3__intnlparam = __const_UnityEngineVector3_0;
                __4__intnlparam = __const_UnityEngineVector3_0;
                __0__intnlparam = false;
            }
            else
            {
                if (__1__intnlparam.transform.childCount < 2)
                {
                    __2__intnlparam = __const_UnityEngineVector3_0;
                    __3__intnlparam = __const_UnityEngineVector3_0;
                    __4__intnlparam = __const_UnityEngineVector3_0;
                    __0__intnlparam = false;
                }
                else
                {
                    __lcl_idHolder_UnityEngineTransform_0 = __1__intnlparam.transform.GetChild(1);
                    if (!VRC.SDKBase.Utilities.IsValid(__lcl_idHolder_UnityEngineTransform_0))
                    {
                        __2__intnlparam = __const_UnityEngineVector3_0;
                        __3__intnlparam = __const_UnityEngineVector3_0;
                        __4__intnlparam = __const_UnityEngineVector3_0;
                        __0__intnlparam = false;
                    }
                    else
                    {
                        __2__intnlparam = __lcl_idHolder_UnityEngineTransform_0.localPosition;
                        __3__intnlparam = __lcl_idHolder_UnityEngineTransform_0.localScale;
                        __4__intnlparam = __lcl_idHolder_UnityEngineTransform_0.localEulerAngles;
                        __0__intnlparam = true;
                    }
                }
            }
            return;
        }
    }
}