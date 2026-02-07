// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.UdonScript
{
    public class QvPen_LateSync : UdonSharpBehaviour
    {
        UnityEngine.Vector3 __0___0_GetCalibrationSignal__ret = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector3[] __1_data__param = null;
        System.Int32 __0_index__param = 0;
        UnityEngine.Color __6__intnlparam = null /* "RGBA(0.000, 0.000, 0.000, 0.000)" */;
        System.Boolean forceStart = false;
        System.String __0___0_ColorBeginTag__ret = null;
        System.Boolean _isNetworkSettled = false;
        UnityEngine.LineRenderer nextInk = null;
        UnityEngine.Vector3 endSignal = null /* "(271828200.00, 0.00, 62831.85)" */;
        VRC.Udon.UdonBehaviour __0_get_pen__ret = null;
        VRC.Udon.UdonBehaviour __0_value__param = null;
        System.String __0_get_logPrefix__ret = null;
        UnityEngine.LineRenderer[] linesBuffer = null /* [] */;
        VRC.Udon.Common.SerializationResult onPostSerializationResult = null /* {"success": false, "byteCount": 0} */;
        UnityEngine.Vector3 __3__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector3 __const_UnityEngineVector3_0 = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector3[] __0_data__param = null;
        System.Boolean __0__intnlparam = false;
        System.String _logPrefix = null;
        UnityEngine.Vector3 __0___0_GetPenIdVector__ret = null /* "(0.00, 0.00, 0.00)" */;
        System.Int32 retryCount = 0;
        UnityEngine.Vector3[] __4_data__param = null;
        UnityEngine.LineRenderer __0_GetNextInk__ret = null;
        UnityEngine.Vector3 __4__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        VRC.SDKBase.VRCPlayerApi onOwnershipTransferredPlayer = null;
        VRC.SDKBase.VRCPlayerApi onPlayerJoinedPlayer = null;
        UnityEngine.Vector3 beginSignal = null /* "(271828200.00, 1.00, 62831.85)" */;
        UnityEngine.Vector3[] __3_data__param = null;
        System.Boolean isInUseSyncBuffer = false;
        UnityEngine.GameObject __1__intnlparam = null;
        VRC.Udon.UdonBehaviour _pen_k__BackingField = null;
        System.Object[] __gintnl_SystemObjectArray_0 = null /* [null, null, null, null, null] */;
        UnityEngine.Transform __0_get_InkPoolSynced__ret = null;
        System.Object __1_o__param = null;
        System.Object __0_o__param = null;
        UnityEngine.Color __0_c__param = null /* "RGBA(0.000, 0.000, 0.000, 0.000)" */;
        System.Object __2_o__param = null;
        UnityEngine.Transform inkPoolSynced = null;
        UnityEngine.Vector3[] __0_get_syncedData__ret = null;
        System.String __5__intnlparam = null;
        UnityEngine.Vector3 errorSignal = null /* "(271828200.00, -1.00, 62831.85)" */;
        UnityEngine.Vector3[] _syncedData = null;
        UnityEngine.Color logColor = null /* "RGBA(0.949, 0.490, 0.290, 1.000)" */;
        UnityEngine.Vector3 __0___0_GetData__ret = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector3[] __2_data__param = null;
        System.Boolean __0_get_isNetworkSettled__ret = false;
        UnityEngine.Vector3[] __1_value__param = null;
        UnityEngine.Vector3 __2__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        System.Int32 inkIndex = -1;
        UnityEngine.Transform inkPoolNotSynced = null;
        VRC.Udon.UdonBehaviour __0_pen__param = null;
        UnityEngine.Transform __0_get_InkPoolNotSynced__ret = null;

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
            System.Boolean __intnl_SystemBoolean_0;
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
            System.Boolean __intnl_SystemBoolean_1;
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
            UnityEngine.Vector3[] __intnl_UnityEngineVector3Array_0;
            System.Boolean __intnl_SystemBoolean_2;
            if (forceStart)
            {
                _syncedData = new UnityEngine.Vector3[](2);
                __intnl_UnityEngineVector3Array_0 = _syncedData;
                get_pen();
                __0_get_pen__ret.SendCustomEvent("get_penIdVector");
                __intnl_UnityEngineVector3Array_0.Set(0, __0_get_pen__ret.GetProgramVariable("__0_get_penIdVector__ret"));
                _syncedData.Set(1, beginSignal);
                __intnl_SystemBoolean_2 = VRC.SDKBase.Networking.IsOwner(this.gameObject);
                if (__intnl_SystemBoolean_2)
                {
                    _RequestSendPackage();
                }
            }
            else
            {
                _syncedData = __1_value__param;
                __intnl_SystemBoolean_2 = VRC.SDKBase.Networking.IsOwner(this.gameObject);
                if (__intnl_SystemBoolean_2)
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
            System.Boolean __intnl_SystemBoolean_3;
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
            System.Boolean __intnl_SystemBoolean_4;
            __intnl_SystemBoolean_4 = VRC.SDKBase.VRCPlayerApi.GetPlayerCount() > 1;
            if (__intnl_SystemBoolean_4)
            {
                __intnl_SystemBoolean_4 = VRC.SDKBase.Networking.IsOwner(this.gameObject);
            }
            if (__intnl_SystemBoolean_4)
            {
                get_isNetworkSettled();
                if (__0_get_isNetworkSettled__ret)
                {
                    isInUseSyncBuffer = true;
                    this.RequestSerialization();
                    return;
                }
                else
                {
                    this.SendCustomEventDelayedSeconds("_RequestSendPackage", 1.84f, null /* 0 */);
                    return;
                }
            }
            else
            {
                goto label_bb_00000660;
            }
        label_bb_00000660:
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
            System.Boolean __intnl_SystemBoolean_13;
            UnityEngine.Vector3 __lcl_ownerIdVector_UnityEngineVector3_0;
            VRC.SDK3.Data.DataToken __lcl_lengthToken_VRCSDK3DataDataToken_0;
            UnityEngine.LineRenderer __lcl_ink_UnityEngineLineRenderer_0;
            System.Int32 __lcl_totalLength_SystemInt32_0;
            System.Int32 __lcl_index_SystemInt32_0;
            VRC.SDK3.Data.DataToken __lcl_dataToken_VRCSDK3DataDataToken_0;
            UnityEngine.Vector3[] __lcl_data_UnityEngineVector3Array_0;
            UnityEngine.Vector3 __lcl__discard_UnityEngineVector3_0;
            VRC.SDK3.Data.DataList __lcl_dataList_VRCSDK3DataDataList_0;
            VRC.SDK3.Data.DataList __lcl_lengthList_VRCSDK3DataDataList_0;
            System.Object __intnl_SystemObject_1;
            System.Int32 __lcl_i_SystemInt32_0;
            UnityEngine.Vector3 __lcl_inkIdVector_UnityEngineVector3_0;
            System.Boolean __intnl_SystemBoolean_12;
            UnityEngine.Vector3[] __lcl_joinedData_UnityEngineVector3Array_0;
            System.Int32 __intnl_SystemInt32_3;
            System.Int32 __intnl_SystemInt32_4;
            System.Int32 __intnl_SystemInt32_7;
            System.Int32 __intnl_SystemInt32_6;
            System.Int32 __intnl_SystemInt32_9;
            System.Int32 __intnl_SystemInt32_8;
            System.Single __intnl_SystemSingle_3;
            System.Int32 __lcl_length_SystemInt32_0;
            UnityEngine.Vector3 __lcl_signal_UnityEngineVector3_0;
            UnityEngine.Vector3 __intnl_UnityEngineVector3_1;
            System.Int32 __lcl_n_SystemInt32_0;
            UnityEngine.Vector3[] __lcl_lengthVectors_UnityEngineVector3Array_0;
            UnityEngine.Vector3[] __intnl_UnityEngineVector3Array_1;
            System.Boolean __intnl_SystemBoolean_6;
            isInUseSyncBuffer = false;
            if (onPostSerializationResult.success)
            {
                retryCount = 0;
                get_syncedData();
                __2_data__param = __0_get_syncedData__ret;
                GetCalibrationSignal();
                __lcl_signal_UnityEngineVector3_0 = __0___0_GetCalibrationSignal__ret;
                __intnl_SystemBoolean_6 = __lcl_signal_UnityEngineVector3_0 == errorSignal;
                if (__intnl_SystemBoolean_6)
                {
                    return;
                }
                else if (__lcl_signal_UnityEngineVector3_0 == beginSignal)
                {
                    forceStart = false;
                    linesBuffer = inkPoolSynced.transform.GetComponentsInChildren(
                        null /* "UnityEngine.LineRenderer, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
                    inkIndex = -1;
                    nextInk = null;
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
                            function_8();
                            __lcl__discard_UnityEngineVector3_0 = __2__intnlparam;
                            __lcl_inkIdVector_UnityEngineVector3_0 = __3__intnlparam;
                            __lcl_ownerIdVector_UnityEngineVector3_0 = __4__intnlparam;
                            if (__0__intnlparam)
                            {
                                get_pen();
                                __0_get_pen__ret.SetProgramVariable("__0_lineRenderer__param", __lcl_ink_UnityEngineLineRenderer_0);
                                __0_get_pen__ret.SetProgramVariable("__2_mode__param", 2);
                                __0_get_pen__ret.SetProgramVariable("__1_inkIdVector__param", __lcl_inkIdVector_UnityEngineVector3_0);
                                __0_get_pen__ret.SetProgramVariable("__1_ownerIdVector__param", __lcl_ownerIdVector_UnityEngineVector3_0);
                                __0_get_pen__ret.SendCustomEvent("__0__PackData");
                                __intnl_SystemObject_1 = __0_get_pen__ret.GetProgramVariable("__0___0__PackData__ret");
                                __intnl_UnityEngineVector3Array_1 = __intnl_SystemObject_1;
                                __lcl_data_UnityEngineVector3Array_0 = __intnl_UnityEngineVector3Array_1;
                                __lcl_length_SystemInt32_0 = __lcl_data_UnityEngineVector3Array_0.Length;
                                __lcl_dataList_VRCSDK3DataDataList_0.Add(new VRC.SDK3.Data.DataToken(__lcl_data_UnityEngineVector3Array_0));
                                __lcl_lengthList_VRCSDK3DataDataList_0.Add((VRC.SDK3.Data.DataToken)__lcl_length_SystemInt32_0);
                                __lcl_totalLength_SystemInt32_0 = __lcl_totalLength_SystemInt32_0 + __lcl_length_SystemInt32_0;
                                function_6();
                                __lcl_ink_UnityEngineLineRenderer_0 = __0_GetNextInk__ret;
                                __intnl_SystemBoolean_12 = VRC.SDKBase.Utilities.IsValid(__lcl_ink_UnityEngineLineRenderer_0);
                                if (__intnl_SystemBoolean_12)
                                {
                                    __intnl_SystemInt32_3 = __lcl_ink_UnityEngineLineRenderer_0.positionCount;
                                    __intnl_SystemInt32_4 = __lcl_totalLength_SystemInt32_0 + __intnl_SystemInt32_3;
                                    __intnl_SystemBoolean_13 = __intnl_SystemInt32_4 > 80;
                                    if (!__intnl_SystemBoolean_13)
                                    {
                                        continue;
                                    }
                                    else
                                    {
                                        nextInk = __lcl_ink_UnityEngineLineRenderer_0;
                                        break;
                                    }
                                }
                                else
                                {
                                    nextInk = null;
                                    break;
                                }
                            }
                            else
                            {
                                function_6();
                                __lcl_ink_UnityEngineLineRenderer_0 = __0_GetNextInk__ret;
                            }
                        }
                        __intnl_SystemInt32_3 = __lcl_lengthList_VRCSDK3DataDataList_0.Count;
                        __intnl_SystemInt32_4 = __intnl_SystemInt32_3 + 2;
                        __lcl_lengthVectors_UnityEngineVector3Array_0 = new UnityEngine.Vector3[](__intnl_SystemInt32_4 / 3);
                        __lcl_i_SystemInt32_0 = 0;
                        __lcl_n_SystemInt32_0 = __lcl_lengthList_VRCSDK3DataDataList_0.Count;
                        __intnl_SystemBoolean_12 = __lcl_i_SystemInt32_0 < __lcl_n_SystemInt32_0;
                        while (__intnl_SystemBoolean_12)
                        {
                            __intnl_SystemBoolean_13 = __lcl_lengthList_VRCSDK3DataDataList_0.TryGetValue(__lcl_i_SystemInt32_0, null /* 6 */,
                                                                                                          out __lcl_lengthToken_VRCSDK3DataDataToken_0);
                            if (__intnl_SystemBoolean_13)
                            {
                                __intnl_SystemInt32_6 = __lcl_i_SystemInt32_0 / 3;
                                __intnl_UnityEngineVector3_1 = __lcl_lengthVectors_UnityEngineVector3Array_0.Get(__intnl_SystemInt32_6);
                                __intnl_SystemInt32_7 = __lcl_i_SystemInt32_0 % 3;
                                __intnl_SystemInt32_8 = __lcl_lengthToken_VRCSDK3DataDataToken_0.Int;
                                __intnl_SystemSingle_3 = System.Convert.ToSingle(__intnl_SystemInt32_8);
                                __intnl_UnityEngineVector3_1.set_Item(__intnl_SystemInt32_7, __intnl_SystemSingle_3);
                                __intnl_SystemInt32_9 = __lcl_i_SystemInt32_0 / 3;
                                __lcl_lengthVectors_UnityEngineVector3Array_0.Set(__intnl_SystemInt32_9, __intnl_UnityEngineVector3_1);
                            }
                            __lcl_i_SystemInt32_0 = __lcl_i_SystemInt32_0 + 1;
                            __intnl_SystemBoolean_12 = __lcl_i_SystemInt32_0 < __lcl_n_SystemInt32_0;
                        }
                        __intnl_SystemInt32_6 = __lcl_lengthVectors_UnityEngineVector3Array_0.Length;
                        __intnl_SystemInt32_7 = 2 + __intnl_SystemInt32_6;
                        __intnl_SystemInt32_8 = __intnl_SystemInt32_7 + __lcl_totalLength_SystemInt32_0;
                        __lcl_joinedData_UnityEngineVector3Array_0 = new UnityEngine.Vector3[](__intnl_SystemInt32_8);
                        __lcl_index_SystemInt32_0 = 0;
                        get_pen();
                        __0_get_pen__ret.SendCustomEvent("get_penIdVector");
                        __intnl_SystemObject_1 = __0_get_pen__ret.GetProgramVariable("__0_get_penIdVector__ret");
                        __intnl_UnityEngineVector3_1 = __intnl_SystemObject_1;
                        __lcl_joinedData_UnityEngineVector3Array_0.Set(0, __intnl_UnityEngineVector3_1);
                        __lcl_index_SystemInt32_0 = __lcl_index_SystemInt32_0 + 1;
                        __intnl_SystemInt32_9 = __lcl_lengthList_VRCSDK3DataDataList_0.Count;
                        __intnl_SystemSingle_3 = System.Convert.ToSingle(__intnl_SystemInt32_9);
                        __lcl_joinedData_UnityEngineVector3Array_0.Set(
                            1,
                            new UnityEngine.Vector3(__intnl_SystemSingle_3, System.Convert.ToSingle(__lcl_joinedData_UnityEngineVector3Array_0.Length), 0.0f));
                        __lcl_index_SystemInt32_0 = __lcl_index_SystemInt32_0 + 1;
                        System.Array.Copy(__lcl_lengthVectors_UnityEngineVector3Array_0, 0, __lcl_joinedData_UnityEngineVector3Array_0,
                                          __lcl_index_SystemInt32_0, __lcl_lengthVectors_UnityEngineVector3Array_0.Length);
                        __lcl_index_SystemInt32_0 = __lcl_index_SystemInt32_0 + __lcl_lengthVectors_UnityEngineVector3Array_0.Length;
                        __lcl_i_SystemInt32_0 = 0;
                        __lcl_n_SystemInt32_0 = __lcl_dataList_VRCSDK3DataDataList_0.Count;
                        __intnl_SystemBoolean_12 = __lcl_i_SystemInt32_0 < __lcl_n_SystemInt32_0;
                        while (__intnl_SystemBoolean_12)
                        {
                            __intnl_SystemBoolean_13 = __lcl_dataList_VRCSDK3DataDataList_0.TryGetValue(__lcl_i_SystemInt32_0, null /* 15 */,
                                                                                                        out __lcl_dataToken_VRCSDK3DataDataToken_0);
                            if (__intnl_SystemBoolean_13)
                            {
                                __lcl_data_UnityEngineVector3Array_0 = __lcl_dataToken_VRCSDK3DataDataToken_0.Reference;
                                System.Array.Copy(__lcl_data_UnityEngineVector3Array_0, 0, __lcl_joinedData_UnityEngineVector3Array_0,
                                                  __lcl_index_SystemInt32_0, __lcl_data_UnityEngineVector3Array_0.Length);
                                __lcl_index_SystemInt32_0 = __lcl_index_SystemInt32_0 + __lcl_data_UnityEngineVector3Array_0.Length;
                            }
                            __lcl_i_SystemInt32_0 = __lcl_i_SystemInt32_0 + 1;
                            __intnl_SystemBoolean_12 = __lcl_i_SystemInt32_0 < __lcl_n_SystemInt32_0;
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
                    return;
                }
                else if (__lcl_signal_UnityEngineVector3_0 == endSignal)
                {
                    linesBuffer = new UnityEngine.LineRenderer[](0);
                    __intnl_UnityEngineVector3Array_1 = new UnityEngine.Vector3[](0);
                    __1_value__param = __intnl_UnityEngineVector3Array_1;
                    function_1();
                    isInUseSyncBuffer = false;
                    return;
                }
                else
                {
                    goto label_bb_00000990;
                }
            }
            else
            {
                __intnl_SystemInt32_3 = retryCount;
                retryCount = __intnl_SystemInt32_3 + 1;
                __intnl_SystemBoolean_6 = __intnl_SystemInt32_3 < 3;
                if (__intnl_SystemBoolean_6)
                {
                    this.SendCustomEventDelayedSeconds("_RequestSendPackage", 1.84f, null /* 0 */);
                }
                goto label_bb_00001518;
            }
        label_bb_00000990:
        label_bb_00001518:
        }

        void function_3()
        {
            System.Int32 __lcl_length_SystemInt32_1;
            System.Int32 __lcl_dataLength_SystemInt32_0;
            System.Int32 __lcl_index_SystemInt32_1;
            System.Boolean __intnl_SystemBoolean_19;
            System.Boolean __intnl_SystemBoolean_14;
            System.Int32 __lcl_check_SystemInt32_0;
            System.Int32 __lcl_i_SystemInt32_1;
            UnityEngine.Vector3 __lcl_signal_UnityEngineVector3_1;
            System.Boolean __intnl_SystemBoolean_15;
            System.Int32 __lcl_currentSyncState_SystemInt32_0;
            UnityEngine.Vector3[] __lcl_stroke_UnityEngineVector3Array_0;
            UnityEngine.Vector3[] __lcl_lengthVectors_UnityEngineVector3Array_1;
            System.Boolean __intnl_SystemBoolean_20;
            UnityEngine.Vector3 __lcl_penIdVector_UnityEngineVector3_0;
            __intnl_SystemBoolean_14 = _syncedData == null;
            if (!__intnl_SystemBoolean_14)
            {
                __intnl_SystemBoolean_14 = _syncedData.Length < 2;
            }
            if (__intnl_SystemBoolean_14)
            {
                return;
            }
            else
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
                    if (__lcl_currentSyncState_SystemInt32_0 == 2)
                    {
                        return;
                    }
                    else
                    {
                        __2_data__param = __1_data__param;
                        GetCalibrationSignal();
                        __lcl_signal_UnityEngineVector3_1 = __0___0_GetCalibrationSignal__ret;
                        if (__lcl_signal_UnityEngineVector3_1 == beginSignal)
                        {
                            __intnl_SystemBoolean_19 = __lcl_currentSyncState_SystemInt32_0 == 0;
                            if (__intnl_SystemBoolean_19)
                            {
                                get_pen();
                                __0_get_pen__ret.SetProgramVariable("currentSyncState", 1);
                            }
                            return;
                        }
                        else
                        {
                            __intnl_SystemBoolean_19 = __lcl_signal_UnityEngineVector3_1 == endSignal;
                            if (__intnl_SystemBoolean_19)
                            {
                                __intnl_SystemBoolean_20 = __lcl_currentSyncState_SystemInt32_0 == 1;
                                if (__intnl_SystemBoolean_20)
                                {
                                    get_pen();
                                    __0_get_pen__ret.SetProgramVariable("currentSyncState", 2);
                                }
                                goto label_bb_00001d94;
                            }
                            else
                            {
                                __intnl_SystemBoolean_20 = __1_data__param.Length > 2;
                                if (__intnl_SystemBoolean_20)
                                {
                                    __lcl_index_SystemInt32_1 = 1;
                                    __lcl_length_SystemInt32_1 =
                                        System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__1_data__param.Get(__lcl_index_SystemInt32_1).x)));
                                    __lcl_check_SystemInt32_0 =
                                        System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__1_data__param.Get(__lcl_index_SystemInt32_1).y)));
                                    __lcl_index_SystemInt32_1 = __lcl_index_SystemInt32_1 + 1;
                                    if (__lcl_check_SystemInt32_0 != __1_data__param.Length)
                                    {
                                        return;
                                    }
                                    else
                                    {
                                        __lcl_lengthVectors_UnityEngineVector3Array_1 = new UnityEngine.Vector3[]((__lcl_length_SystemInt32_1 + 2) / 3);
                                        System.Array.Copy(__1_data__param, __lcl_index_SystemInt32_1, __lcl_lengthVectors_UnityEngineVector3Array_1, 0,
                                                          __lcl_lengthVectors_UnityEngineVector3Array_1.Length);
                                        __lcl_index_SystemInt32_1 = __lcl_index_SystemInt32_1 + __lcl_lengthVectors_UnityEngineVector3Array_1.Length;
                                        __lcl_i_SystemInt32_1 = 0;
                                        if (__lcl_i_SystemInt32_1 < __lcl_length_SystemInt32_1)
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
                                            goto label_bb_00001ba4;
                                        }
                                        else
                                        {
                                            goto label_bb_00001d94;
                                        }
                                    }
                                }
                                else
                                {
                                    goto label_bb_00001d94;
                                }
                            }
                        }
                    }
                }
                else
                {
                    goto label_bb_00001d94;
                }
            }
        label_bb_00001ba4:
        label_bb_00001d94:
        }

        void function_4()
        {
            get_pen();
            __0_get_pen__ret.SendCustomEvent("get_penIdVector");
            new UnityEngine.Vector3[](2).Set(0, __0_get_pen__ret.GetProgramVariable("__0_get_penIdVector__ret"));
            new UnityEngine.Vector3[](2).Set(1, beginSignal);
            __0_data__param = new UnityEngine.Vector3[](2);
            function_2();
            return;
        }

        void function_5()
        {
            get_pen();
            __0_get_pen__ret.SendCustomEvent("get_penIdVector");
            new UnityEngine.Vector3[](2).Set(0, __0_get_pen__ret.GetProgramVariable("__0_get_penIdVector__ret"));
            new UnityEngine.Vector3[](2).Set(1, endSignal);
            __0_data__param = new UnityEngine.Vector3[](2);
            function_2();
            return;
        }

        void GetCalibrationSignal()
        {
            System.Boolean __intnl_SystemBoolean_23;
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
            System.Boolean __intnl_SystemBoolean_24;
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
            System.Boolean __intnl_SystemBoolean_25;
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
            UnityEngine.LineRenderer __lcl_ink_UnityEngineLineRenderer_1;
            inkIndex = UnityEngine.Mathf.Max(-1, inkIndex);
            inkIndex = inkIndex + 1;
            while (inkIndex < linesBuffer.Length)
            {
                __lcl_ink_UnityEngineLineRenderer_1 = linesBuffer.Get(inkIndex);
                if (VRC.SDKBase.Utilities.IsValid(__lcl_ink_UnityEngineLineRenderer_1))
                {
                    __0_GetNextInk__ret = __lcl_ink_UnityEngineLineRenderer_1;
                    return;
                }
                inkIndex = inkIndex + 1;
            }
            __0_GetNextInk__ret = null;
            return;
        }

        void function_7()
        {
            __6__intnlparam = __0_c__param;
            function_9();
            __0___0_ColorBeginTag__ret = System.String.Format("<color=\"#{0}\">", __5__intnlparam);
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
                function_7();
                __gintnl_SystemObjectArray_0.Set(0, __0___0_ColorBeginTag__ret);
                __gintnl_SystemObjectArray_0.Set(1, "QvPen");
                __gintnl_SystemObjectArray_0.Set(2, "Udon");
                __gintnl_SystemObjectArray_0.Set(3, "QvPen_LateSync");
                __gintnl_SystemObjectArray_0.Set(4, "</color>");
                _logPrefix = System.String.Format("[{0}{1}.{2}.{3}{4}] ", __gintnl_SystemObjectArray_0);
                __0_get_logPrefix__ret = _logPrefix;
            }
            return;
        }

        void function_8()
        {
            UnityEngine.Transform __lcl_idHolder_UnityEngineTransform_0;
            if (VRC.SDKBase.Utilities.IsValid(__1__intnlparam))
            {
                if (__1__intnlparam.transform.childCount < 2)
                {
                    __2__intnlparam = __const_UnityEngineVector3_0;
                    __3__intnlparam = __const_UnityEngineVector3_0;
                    __4__intnlparam = __const_UnityEngineVector3_0;
                    __0__intnlparam = false;
                    return;
                }
                else
                {
                    __lcl_idHolder_UnityEngineTransform_0 = __1__intnlparam.transform.GetChild(1);
                    if (VRC.SDKBase.Utilities.IsValid(__lcl_idHolder_UnityEngineTransform_0))
                    {
                        __2__intnlparam = __lcl_idHolder_UnityEngineTransform_0.localPosition;
                        __3__intnlparam = __lcl_idHolder_UnityEngineTransform_0.localScale;
                        __4__intnlparam = __lcl_idHolder_UnityEngineTransform_0.localEulerAngles;
                        __0__intnlparam = true;
                        return;
                    }
                    else
                    {
                        __2__intnlparam = __const_UnityEngineVector3_0;
                        __3__intnlparam = __const_UnityEngineVector3_0;
                        __4__intnlparam = __const_UnityEngineVector3_0;
                        __0__intnlparam = false;
                        return;
                    }
                }
            }
            else
            {
                __2__intnlparam = __const_UnityEngineVector3_0;
                __3__intnlparam = __const_UnityEngineVector3_0;
                __4__intnlparam = __const_UnityEngineVector3_0;
                __0__intnlparam = false;
                return;
            }
        }

        void function_9()
        {
            __6__intnlparam = __6__intnlparam * 255.0f;
            __5__intnlparam = System.String.Format("{0:x2}{1:x2}{2:x2}", UnityEngine.Mathf.RoundToInt(__6__intnlparam.r),
                                                   UnityEngine.Mathf.RoundToInt(__6__intnlparam.g), UnityEngine.Mathf.RoundToInt(__6__intnlparam.b));
            return;
        }
    }
}