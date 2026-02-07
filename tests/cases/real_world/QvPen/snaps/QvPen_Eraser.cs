// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.UdonScript
{
    public class QvPen_Eraser : UdonSharpBehaviour
    {
        System.Single __0_get_eraserRadius__ret = 0.0f;
        UnityEngine.Vector3 __13__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector3[] tmpErasedData = null;
        UnityEngine.Vector3 __6__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Renderer __0_get_renderer__ret = null;
        UnityEngine.Vector3 __8__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        System.String __0___0_ColorBeginTag__ret = null;
        System.Int32 __10__intnlparam = 0;
        UnityEngine.SphereCollider __0_get_sphereCollider__ret = null;
        UnityEngine.Vector3[] __5_data__param = null;
        UnityEngine.Renderer _renderer = null;
        System.String __0_get_logPrefix__ret = null;
        VRC.Udon.UdonBehaviour eraserManager = null;
        System.Int32 __19__intnlparam = 0;
        UnityEngine.Component __3__intnlparam = null;
        System.Int32 inkColliderLayer = 0;
        VRC.SDK3.Components.VRCPickup __0_get_pickup__ret = null;
        UnityEngine.Vector3 __16__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        System.Boolean isUser = false;
        System.Boolean __0_get_isHeld__ret = false;
        System.Int32 __0_get_localPlayerId__ret = 0;
        UnityEngine.Vector3 __const_UnityEngineVector3_0 = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Transform __0__intnlparam = null;
        UnityEngine.Vector3 __7__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        System.String _logPrefix = null;
        System.Int32 __9__intnlparam = 0;
        System.Single[] __gintnl_SystemSingleArray_0 = null /* [0.0, 0.0, 0.0] */;
        System.Boolean isErasing = false;
        UnityEngine.Vector3[] __15__intnlparam = null;
        UnityEngine.Vector3[] __4_data__param = null;
        UnityEngine.Vector3 __20__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        System.Boolean __4__intnlparam = false;
        System.Boolean isPickedUp = false;
        System.Int32 __12__intnlparam = 0;
        VRC.Udon.UdonBehaviour manager = null;
        VRC.SDKBase.VRCPlayerApi __0_get_localPlayer__ret = null;
        System.Boolean _isCheckedEraserRadius = false;
        VRC.SDKBase.VRCPlayerApi onPlayerJoinedPlayer = null;
        UnityEngine.Material erasing = null;
        System.Boolean __0_get_IsUser__ret = false;
        UnityEngine.Vector3[] __3_data__param = null;
        UnityEngine.Transform __1__intnlparam = null;
        System.Int32 __18__intnlparam = 0;
        UnityEngine.Vector3 __const_UnityEngineVector3_1 = null /* "(1.00, 1.00, 1.00)" */;
        UnityEngine.Vector3[] __11__intnlparam = null;
        System.Object[] __gintnl_SystemObjectArray_0 = null /* [null, null, null, null, null] */;
        UnityEngine.SphereCollider _sphereCollider = null;
        System.Object __1_o__param = null;
        System.Object __0_o__param = null;
        UnityEngine.Color __0_c__param = null /* "RGBA(0.000, 0.000, 0.000, 0.000)" */;
        System.Object __2_o__param = null;
        VRC.SDK3.Components.VRCObjectSync __0_get_objectSync__ret = null;
        VRC.SDK3.Components.VRCPickup _pickup = null;
        System.Single _eraserRadius = 0.0f;
        UnityEngine.GameObject __5__intnlparam = null;
        VRC.Udon.UdonBehaviour __0_eraserManager__param = null;
        UnityEngine.Vector3[] __17__intnlparam = null;
        UnityEngine.Transform inkPoolRoot = null;
        UnityEngine.Color __22__intnlparam = null /* "RGBA(0.000, 0.000, 0.000, 0.000)" */;
        UnityEngine.Color logColor = null /* "RGBA(0.949, 0.490, 0.290, 1.000)" */;
        System.Int32 __14__intnlparam = 0;
        UnityEngine.Quaternion __const_UnityEngineQuaternion_0 = null /* "(0.00000, 0.00000, 0.00000, 1.00000)" */;
        UnityEngine.Material normal = null;
        VRC.SDKBase.VRCPlayerApi _localPlayer = null;
        VRC.Udon.UdonBehaviour __2__intnlparam = null;
        VRC.SDK3.Components.VRCObjectSync _objectSync = null;
        UnityEngine.Collider[] results4 = null /* [null, null, null, null] */;
        System.String __21__intnlparam = null;

        void get_renderer()
        {
            if ((UnityEngine.Object)_renderer)
            {
                __0_get_renderer__ret = _renderer;
            }
            else
            {
                _renderer = this.transform.GetComponentInChildren(
                    true, null /* "UnityEngine.Renderer, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
                __0_get_renderer__ret = _renderer;
            }
            return;
        }

        void get_sphereCollider()
        {
            if ((UnityEngine.Object)_sphereCollider)
            {
                __0_get_sphereCollider__ret = _sphereCollider;
            }
            else
            {
                _sphereCollider = this.transform.GetComponent(
                    null /* "UnityEngine.SphereCollider, UnityEngine.PhysicsModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
                __0_get_sphereCollider__ret = _sphereCollider;
            }
            return;
        }

        void get_pickup()
        {
            if ((UnityEngine.Object)_pickup)
            {
                __0_get_pickup__ret = _pickup;
            }
            else
            {
                _pickup = this.GetComponent(null /* "VRC.SDK3.Components.VRCPickup, VRCSDK3, Version=1.0.0.0, Culture=neutral, PublicKeyToken=null" */);
                __0_get_pickup__ret = _pickup;
            }
            return;
        }

        void get_objectSync()
        {
            if ((UnityEngine.Object)_objectSync)
            {
                __0_get_objectSync__ret = _objectSync;
            }
            else
            {
                _objectSync = this.GetComponent(null /* "VRC.SDK3.Components.VRCObjectSync, VRCSDK3, Version=1.0.0.0, Culture=neutral, PublicKeyToken=null" */);
                __0_get_objectSync__ret = _objectSync;
            }
            return;
        }

        public void get_IsUser()
        {
            __0_get_IsUser__ret = isUser;
            return;
        }

        void get_eraserRadius()
        {
            UnityEngine.Vector3 __lcl_s_UnityEngineVector3_0;
            if (_isCheckedEraserRadius)
            {
                __0_get_eraserRadius__ret = _eraserRadius;
                return;
            }
            else
            {
                __lcl_s_UnityEngineVector3_0 = this.transform.lossyScale;
                __gintnl_SystemSingleArray_0.Set(0, __lcl_s_UnityEngineVector3_0.x);
                __gintnl_SystemSingleArray_0.Set(1, __lcl_s_UnityEngineVector3_0.y);
                __gintnl_SystemSingleArray_0.Set(2, __lcl_s_UnityEngineVector3_0.z);
                get_sphereCollider();
                _eraserRadius = UnityEngine.Mathf.Min(__gintnl_SystemSingleArray_0) * __0_get_sphereCollider__ret.radius;
                _isCheckedEraserRadius = true;
                __0_get_eraserRadius__ret = _eraserRadius;
                return;
            }
        }

        void get_localPlayer()
        {
            __0_get_localPlayer__ret = _localPlayer;
            if (__0_get_localPlayer__ret == null)
            {
                _localPlayer = VRC.SDKBase.Networking.LocalPlayer;
                __0_get_localPlayer__ret = _localPlayer;
            }
            return;
        }

        void get_localPlayerId()
        {
            get_localPlayer();
            if (VRC.SDKBase.Utilities.IsValid(__0_get_localPlayer__ret))
            {
                get_localPlayer();
                __0_get_localPlayerId__ret = __0_get_localPlayer__ret.playerId;
            }
            else
            {
                __0_get_localPlayerId__ret = -1;
            }
            return;
        }

        public void __0__Init()
        {
            UnityEngine.Transform __intnl_UnityEngineTransform_0;
            UnityEngine.GameObject __intnl_UnityEngineGameObject_0;
            UnityEngine.GameObject __lcl_inkPoolRootGO_UnityEngineGameObject_0;
            eraserManager = __0_eraserManager__param;
            inkColliderLayer = System.Convert.ToInt32(__0_eraserManager__param.GetProgramVariable("inkColliderLayer"));
            __lcl_inkPoolRootGO_UnityEngineGameObject_0 = UnityEngine.GameObject.Find(System.String.Format("/{0}", "QvPen_Objects"));
            if (VRC.SDKBase.Utilities.IsValid(__lcl_inkPoolRootGO_UnityEngineGameObject_0))
            {
                __intnl_UnityEngineGameObject_0 = inkPoolRoot.gameObject;
                __intnl_UnityEngineGameObject_0.SetActive(false);
                inkPoolRoot = __lcl_inkPoolRootGO_UnityEngineGameObject_0.transform;
            }
            else
            {
                inkPoolRoot.name = "QvPen_Objects";
                __intnl_UnityEngineTransform_0 = null;
                __0__intnlparam = inkPoolRoot;
                __1__intnlparam = __intnl_UnityEngineTransform_0;
                function_2();
                inkPoolRoot.SetAsFirstSibling();
                __intnl_UnityEngineGameObject_0 = inkPoolRoot.gameObject;
                __intnl_UnityEngineGameObject_0.SetActive(true);
            }
            __intnl_UnityEngineTransform_0 = inkPoolRoot.transform;
            __3__intnlparam = __intnl_UnityEngineTransform_0;
            function_3();
            manager = __2__intnlparam;
            if (!VRC.SDKBase.Utilities.IsValid(erasing))
            {
                get_renderer();
                erasing = __0_get_renderer__ret.sharedMaterial;
            }
            get_pickup();
            __0_get_pickup__ret.InteractionText = "Eraser";
            get_pickup();
            __0_get_pickup__ret.UseText = "Erase";
            return;
        }

        public void _onPlayerJoined()
        {
            System.Boolean __intnl_SystemBoolean_8;
            __intnl_SystemBoolean_8 = isUser;
            if (__intnl_SystemBoolean_8)
            {
                __intnl_SystemBoolean_8 = isErasing;
            }
            if (__intnl_SystemBoolean_8)
            {
                this.SendCustomNetworkEvent(null /* 0 */, "OnPickupEvent");
            }
            return;
        }

        public void _onPickup()
        {
            isUser = true;
            get_sphereCollider();
            __0_get_sphereCollider__ret.enabled = false;
            eraserManager.SendCustomEvent("_TakeOwnership");
            eraserManager.SendCustomNetworkEvent(null /* 0 */, "StartUsing");
            this.SendCustomNetworkEvent(null /* 0 */, "OnPickupEvent");
            return;
        }

        public void _onDrop()
        {
            isUser = false;
            get_sphereCollider();
            __0_get_sphereCollider__ret.enabled = true;
            eraserManager.SendCustomEvent("_ClearSyncBuffer");
            eraserManager.SendCustomNetworkEvent(null /* 0 */, "EndUsing");
            this.SendCustomNetworkEvent(null /* 0 */, "OnDropEvent");
            return;
        }

        public void _onPickupUseDown()
        {
            this.SendCustomNetworkEvent(null /* 0 */, "StartErasing");
            return;
        }

        public void _onPickupUseUp()
        {
            this.SendCustomNetworkEvent(null /* 0 */, "FinishErasing");
            return;
        }

        public void OnPickupEvent()
        {
            get_renderer();
            __0_get_renderer__ret.sharedMaterial = normal;
            return;
        }

        public void OnDropEvent()
        {
            get_renderer();
            __0_get_renderer__ret.sharedMaterial = erasing;
            return;
        }

        public void StartErasing()
        {
            isErasing = true;
            get_renderer();
            __0_get_renderer__ret.sharedMaterial = erasing;
            return;
        }

        public void FinishErasing()
        {
            isErasing = false;
            get_renderer();
            __0_get_renderer__ret.sharedMaterial = normal;
            return;
        }

        public void __0__SendData()
        {
            eraserManager.SetProgramVariable("__0_data__param", __3_data__param);
            eraserManager.SendCustomEvent("__0__SendData");
            return;
        }

        public void _postLateUpdate()
        {
            System.Boolean __intnl_SystemBoolean_13;
            System.Boolean __intnl_SystemBoolean_16;
            UnityEngine.Vector3[] __lcl_data_UnityEngineVector3Array_0;
            System.Boolean __intnl_SystemBoolean_11;
            UnityEngine.LineRenderer __lcl_lineRenderer_UnityEngineLineRenderer_0;
            UnityEngine.Vector3 __lcl__discard_UnityEngineVector3_0;
            System.Int32 __lcl_count_SystemInt32_0;
            System.Boolean __intnl_SystemBoolean_14;
            System.Int32 __lcl_i_SystemInt32_0;
            UnityEngine.Vector3 __lcl_inkIdVector_UnityEngineVector3_0;
            System.Boolean __intnl_SystemBoolean_15;
            UnityEngine.Transform __lcl_t_UnityEngineTransform_0;
            UnityEngine.Collider __lcl_other_UnityEngineCollider_0;
            System.Boolean __intnl_SystemBoolean_10;
            UnityEngine.Vector3 __lcl_penIdVector_UnityEngineVector3_0;
            __intnl_SystemBoolean_11 = !isUser;
            if (!__intnl_SystemBoolean_11)
            {
                get_isHeld();
                __intnl_SystemBoolean_11 = !__0_get_isHeld__ret;
            }
            __intnl_SystemBoolean_10 = __intnl_SystemBoolean_11;
            if (!__intnl_SystemBoolean_10)
            {
                __intnl_SystemBoolean_10 = !isErasing;
            }
            if (__intnl_SystemBoolean_10)
            {
                return;
            }
            else
            {
                get_eraserRadius();
                __lcl_count_SystemInt32_0 = UnityEngine.Physics.OverlapSphereNonAlloc(this.transform.position, __0_get_eraserRadius__ret, results4,
                                                                                      1 << inkColliderLayer, null /* 1 */);
                __lcl_i_SystemInt32_0 = 0;
                while (__lcl_i_SystemInt32_0 < __lcl_count_SystemInt32_0)
                {
                    __lcl_other_UnityEngineCollider_0 = results4.Get(__lcl_i_SystemInt32_0);
                    __intnl_SystemBoolean_14 = VRC.SDKBase.Utilities.IsValid(__lcl_other_UnityEngineCollider_0);
                    if (__intnl_SystemBoolean_14)
                    {
                        __lcl_t_UnityEngineTransform_0 = __lcl_other_UnityEngineCollider_0.transform.parent;
                        __intnl_SystemBoolean_14 = VRC.SDKBase.Utilities.IsValid(__lcl_t_UnityEngineTransform_0);
                    }
                    __intnl_SystemBoolean_13 = __intnl_SystemBoolean_14;
                    if (__intnl_SystemBoolean_13)
                    {
                        __intnl_SystemBoolean_13 = VRC.SDKBase.Utilities.IsValid(__lcl_t_UnityEngineTransform_0.parent);
                    }
                    if (__intnl_SystemBoolean_13)
                    {
                        __lcl_lineRenderer_UnityEngineLineRenderer_0 = __lcl_other_UnityEngineCollider_0.transform.GetComponentInParent(
                            null /* "UnityEngine.LineRenderer, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
                        __intnl_SystemBoolean_16 = VRC.SDKBase.Utilities.IsValid(__lcl_lineRenderer_UnityEngineLineRenderer_0);
                        if (__intnl_SystemBoolean_16)
                        {
                            __intnl_SystemBoolean_16 = __lcl_lineRenderer_UnityEngineLineRenderer_0.positionCount > 0;
                        }
                        __intnl_SystemBoolean_15 = __intnl_SystemBoolean_16;
                        if (__intnl_SystemBoolean_15)
                        {
                            __5__intnlparam = __lcl_lineRenderer_UnityEngineLineRenderer_0.gameObject;
                            __6__intnlparam = __lcl_penIdVector_UnityEngineVector3_0;
                            __7__intnlparam = __lcl_inkIdVector_UnityEngineVector3_0;
                            __8__intnlparam = __lcl__discard_UnityEngineVector3_0;
                            function_4();
                            __lcl_penIdVector_UnityEngineVector3_0 = __6__intnlparam;
                            __lcl_inkIdVector_UnityEngineVector3_0 = __7__intnlparam;
                            __lcl__discard_UnityEngineVector3_0 = __8__intnlparam;
                            __intnl_SystemBoolean_15 = __4__intnlparam;
                        }
                        if (__intnl_SystemBoolean_15)
                        {
                            __10__intnlparam = 3;
                            function_5();
                            __lcl_data_UnityEngineVector3Array_0 = new UnityEngine.Vector3[](__9__intnlparam);
                            get_localPlayerId();
                            __10__intnlparam = 3;
                            function_5();
                            __11__intnlparam = __lcl_data_UnityEngineVector3Array_0;
                            __12__intnlparam = 0;
                            __13__intnlparam =
                                new UnityEngine.Vector3(System.Convert.ToSingle(__0_get_localPlayerId__ret), 3.0f, System.Convert.ToSingle(__9__intnlparam));
                            function_6();
                            __11__intnlparam = __lcl_data_UnityEngineVector3Array_0;
                            __12__intnlparam = 1;
                            __13__intnlparam = __lcl_penIdVector_UnityEngineVector3_0;
                            function_6();
                            __11__intnlparam = __lcl_data_UnityEngineVector3Array_0;
                            __12__intnlparam = 2;
                            __13__intnlparam = __lcl_inkIdVector_UnityEngineVector3_0;
                            function_6();
                            __3_data__param = __lcl_data_UnityEngineVector3Array_0;
                            __0__SendData();
                        }
                    }
                    results4.Set(__lcl_i_SystemInt32_0, null);
                    __lcl_i_SystemInt32_0 = __lcl_i_SystemInt32_0 + 1;
                }
                return;
            }
        }

        public void __0__UnpackData()
        {
            System.Int32 __lcl_mode_SystemInt32_0;
            System.Boolean __intnl_SystemBoolean_18;
            __15__intnlparam = __4_data__param;
            function_7();
            __lcl_mode_SystemInt32_0 = __14__intnlparam;
            if (__lcl_mode_SystemInt32_0 == 3)
            {
                __intnl_SystemBoolean_18 = isUser;
                if (__intnl_SystemBoolean_18)
                {
                    __intnl_SystemBoolean_18 = VRC.SDKBase.VRCPlayerApi.GetPlayerCount() > 1;
                }
                if (__intnl_SystemBoolean_18)
                {
                    tmpErasedData = __4_data__param;
                }
                else
                {
                    __5_data__param = __4_data__param;
                    function_0();
                }
            }
            return;
        }

        public void ExecuteEraseInk()
        {
            if (tmpErasedData != null)
            {
                __5_data__param = tmpErasedData;
                function_0();
            }
            tmpErasedData = null;
            return;
        }

        void function_0()
        {
            System.Int32 __lcl_inkId_SystemInt32_0;
            System.Int32 __lcl_penId_SystemInt32_0;
            UnityEngine.Vector3 __lcl_inkIdVector_UnityEngineVector3_1;
            UnityEngine.Vector3 __lcl_penIdVector_UnityEngineVector3_1;
            System.Boolean __intnl_SystemBoolean_20;
            __intnl_SystemBoolean_20 = __5_data__param == null;
            if (!__intnl_SystemBoolean_20)
            {
                __10__intnlparam = 3;
                function_5();
                __intnl_SystemBoolean_20 = __5_data__param.Length < __9__intnlparam;
            }
            if (__intnl_SystemBoolean_20)
            {
                return;
            }
            else
            {
                __17__intnlparam = __5_data__param;
                __18__intnlparam = 1;
                function_8();
                __lcl_penIdVector_UnityEngineVector3_1 = __16__intnlparam;
                __17__intnlparam = __5_data__param;
                __18__intnlparam = 2;
                function_8();
                __lcl_inkIdVector_UnityEngineVector3_1 = __16__intnlparam;
                __20__intnlparam = __lcl_penIdVector_UnityEngineVector3_1;
                function_9();
                __lcl_penId_SystemInt32_0 = __19__intnlparam;
                __20__intnlparam = __lcl_inkIdVector_UnityEngineVector3_1;
                function_9();
                __lcl_inkId_SystemInt32_0 = __19__intnlparam;
                manager.SetProgramVariable("__3_penId__param", __lcl_penId_SystemInt32_0);
                manager.SetProgramVariable("__2_inkId__param", __lcl_inkId_SystemInt32_0);
                manager.SendCustomEvent("__0_RemoveInk");
                return;
            }
        }

        public void get_isHeld()
        {
            __0_get_isHeld__ret = isPickedUp;
            return;
        }

        public void _Respawn()
        {
            get_pickup();
            __0_get_pickup__ret.Drop();
            if (VRC.SDKBase.Networking.LocalPlayer.IsOwner(this.gameObject))
            {
                get_objectSync();
                __0_get_objectSync__ret.Respawn();
            }
            return;
        }

        void function_1()
        {
            __22__intnlparam = __0_c__param;
            function_10();
            __0___0_ColorBeginTag__ret = System.String.Format("<color=\"#{0}\">", __21__intnlparam);
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
                function_1();
                __gintnl_SystemObjectArray_0.Set(0, __0___0_ColorBeginTag__ret);
                __gintnl_SystemObjectArray_0.Set(1, "QvPen");
                __gintnl_SystemObjectArray_0.Set(2, "Udon");
                __gintnl_SystemObjectArray_0.Set(3, "QvPen_Eraser");
                __gintnl_SystemObjectArray_0.Set(4, "</color>");
                _logPrefix = System.String.Format("[{0}{1}.{2}.{3}{4}] ", __gintnl_SystemObjectArray_0);
                __0_get_logPrefix__ret = _logPrefix;
            }
            return;
        }

        void function_2()
        {
            if (VRC.SDKBase.Utilities.IsValid(__0__intnlparam))
            {
                __0__intnlparam.SetParent(__1__intnlparam);
                __0__intnlparam.SetLocalPositionAndRotation(__const_UnityEngineVector3_0, __const_UnityEngineQuaternion_0);
                __0__intnlparam.localScale = __const_UnityEngineVector3_1;
                return;
            }
            else
            {
                return;
            }
        }

        void function_3()
        {
            VRC.Udon.UdonBehaviour __lcl_behaviour_VRCUdonUdonBehaviour_0;
            System.Int32 __intnl_SystemInt32_5;
            System.Int64 __lcl_targetID_SystemInt64_0;
            System.Object __lcl_idValue_SystemObject_0;
            UnityEngine.Component[] __lcl_udonBehaviours_UnityEngineComponentArray_0;
            System.Boolean __intnl_SystemBoolean_28;
            __lcl_udonBehaviours_UnityEngineComponentArray_0 =
                __3__intnlparam.GetComponents(null /* "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
            __lcl_targetID_SystemInt64_0 = 8320864560273335438;
            __intnl_SystemInt32_5 = 0;
            while (__intnl_SystemInt32_5 < __lcl_udonBehaviours_UnityEngineComponentArray_0.Length)
            {
                __lcl_behaviour_VRCUdonUdonBehaviour_0 = __lcl_udonBehaviours_UnityEngineComponentArray_0.Get(__intnl_SystemInt32_5);
                if (__lcl_behaviour_VRCUdonUdonBehaviour_0.GetProgramVariableType("__refl_typeid") == null)
                {
                    goto label_bb_00001ddc;
                }
                else
                {
                    __lcl_idValue_SystemObject_0 = __lcl_behaviour_VRCUdonUdonBehaviour_0.GetProgramVariable("__refl_typeid");
                    __intnl_SystemBoolean_28 = __lcl_idValue_SystemObject_0 != null;
                    if (__intnl_SystemBoolean_28)
                    {
                        __intnl_SystemBoolean_28 = System.Convert.ToInt64(__lcl_idValue_SystemObject_0) == __lcl_targetID_SystemInt64_0;
                    }
                    if (__intnl_SystemBoolean_28)
                    {
                        __2__intnlparam = __lcl_behaviour_VRCUdonUdonBehaviour_0;
                        return;
                    }
                    else
                    {
                        goto label_bb_00001ddc;
                    }
                }
            }
            __2__intnlparam = null;
            return;
        label_bb_00001ddc:
            __intnl_SystemInt32_5 = __intnl_SystemInt32_5 + 1;
            goto label_bb_00001c4c;
        label_bb_00001c4c:
        }

        void function_4()
        {
            UnityEngine.Transform __lcl_idHolder_UnityEngineTransform_0;
            if (VRC.SDKBase.Utilities.IsValid(__5__intnlparam))
            {
                if (__5__intnlparam.transform.childCount < 2)
                {
                    __6__intnlparam = __const_UnityEngineVector3_0;
                    __7__intnlparam = __const_UnityEngineVector3_0;
                    __8__intnlparam = __const_UnityEngineVector3_0;
                    __4__intnlparam = false;
                    return;
                }
                else
                {
                    __lcl_idHolder_UnityEngineTransform_0 = __5__intnlparam.transform.GetChild(1);
                    if (VRC.SDKBase.Utilities.IsValid(__lcl_idHolder_UnityEngineTransform_0))
                    {
                        __6__intnlparam = __lcl_idHolder_UnityEngineTransform_0.localPosition;
                        __7__intnlparam = __lcl_idHolder_UnityEngineTransform_0.localScale;
                        __8__intnlparam = __lcl_idHolder_UnityEngineTransform_0.localEulerAngles;
                        __4__intnlparam = true;
                        return;
                    }
                    else
                    {
                        __6__intnlparam = __const_UnityEngineVector3_0;
                        __7__intnlparam = __const_UnityEngineVector3_0;
                        __8__intnlparam = __const_UnityEngineVector3_0;
                        __4__intnlparam = false;
                        return;
                    }
                }
            }
            else
            {
                __6__intnlparam = __const_UnityEngineVector3_0;
                __7__intnlparam = __const_UnityEngineVector3_0;
                __8__intnlparam = __const_UnityEngineVector3_0;
                __4__intnlparam = false;
                return;
            }
        }

        void function_5()
        {
            switch (__10__intnlparam)
            {
                case 2:
                    __9__intnlparam = 5;
                    return;
                case 3:
                    __9__intnlparam = 4;
                    return;
                case 0:
                    __9__intnlparam = 0;
                    return;
                default:
                    __9__intnlparam = 0;
                    return;
            }
        }

        void function_6()
        {
            System.Boolean __intnl_SystemBoolean_35;
            __intnl_SystemBoolean_35 = __11__intnlparam != null;
            if (__intnl_SystemBoolean_35)
            {
                __intnl_SystemBoolean_35 = __11__intnlparam.Length > __12__intnlparam;
            }
            if (__intnl_SystemBoolean_35)
            {
                __11__intnlparam.Set(__11__intnlparam.Length - 1 - __12__intnlparam, __13__intnlparam);
            }
            return;
        }

        void function_7()
        {
            System.Boolean __intnl_SystemBoolean_36;
            __intnl_SystemBoolean_36 = __15__intnlparam != null;
            if (__intnl_SystemBoolean_36)
            {
                __intnl_SystemBoolean_36 = __15__intnlparam.Length > 0;
            }
            if (__intnl_SystemBoolean_36)
            {
                __17__intnlparam = __15__intnlparam;
                __18__intnlparam = 0;
                function_8();
                __14__intnlparam = System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__16__intnlparam.y)));
            }
            else
            {
                __14__intnlparam = 0;
            }
            return;
        }

        void function_8()
        {
            System.Boolean __intnl_SystemBoolean_37;
            __intnl_SystemBoolean_37 = __17__intnlparam != null;
            if (__intnl_SystemBoolean_37)
            {
                __intnl_SystemBoolean_37 = __17__intnlparam.Length > __18__intnlparam;
            }
            if (__intnl_SystemBoolean_37)
            {
                __16__intnlparam = __17__intnlparam.Get(__17__intnlparam.Length - 1 - __18__intnlparam);
            }
            else
            {
                __16__intnlparam = __const_UnityEngineVector3_0;
            }
            return;
        }

        void function_9()
        {
            __19__intnlparam = (System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__20__intnlparam.x))) & 255) << 24 |
                               (System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__20__intnlparam.y))) & 4095) << 12 |
                               System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__20__intnlparam.z))) & 4095;
            return;
        }

        void function_10()
        {
            __22__intnlparam = __22__intnlparam * 255.0f;
            __21__intnlparam = System.String.Format("{0:x2}{1:x2}{2:x2}", UnityEngine.Mathf.RoundToInt(__22__intnlparam.r),
                                                    UnityEngine.Mathf.RoundToInt(__22__intnlparam.g), UnityEngine.Mathf.RoundToInt(__22__intnlparam.b));
            return;
        }
    }
}