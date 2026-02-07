// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.UdonScript
{
    public class QvPen_Pen : UdonSharpBehaviour
    {
        UnityEngine.Vector3 __35__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        System.Int32 __13__intnlparam = 0;
        System.Single _pointerRadius = 0.0f;
        System.Boolean allowCallPen = false;
        System.Int32 currentState = 0;
        VRC.Udon.UdonBehaviour __6__intnlparam = null;
        System.String __8__intnlparam = null;
        UnityEngine.Renderer pointerRenderer = null;
        VRC.Udon.UdonBehaviour syncer = null;
        UnityEngine.LineRenderer inkPrefab = null;
        UnityEngine.GameObject __32__intnlparam = null;
        UnityEngine.Vector3 __27__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        System.Boolean enabledLateSync = true;
        UnityEngine.Collider onTriggerEnterOther = null;
        UnityEngine.Vector3 __0_ownerIdVector__param = null /* "(0.00, 0.00, 0.00)" */;
        System.String __0___0_ColorBeginTag__ret = null;
        UnityEngine.Vector3 prevClickPos = null /* "(0.00, 0.00, 0.00)" */;
        System.Boolean _isCheckedLocalPlayerId = false;
        System.Int32 __1_inkId__param = 0;
        UnityEngine.Transform inkPosition = null;
        System.Int32 _localPlayerId = 0;
        UnityEngine.Vector3[] __0___0_PackData__ret = null;
        System.Int32 __10__intnlparam = 0;
        UnityEngine.Material pointerMaterialActive = null;
        UnityEngine.Vector3[] __5_data__param = null;
        UnityEngine.Vector3 __0_value__param = null /* "(0.00, 0.00, 0.00)" */;
        System.String __0_get_logPrefix__ret = null;
        System.Int32 __19__intnlparam = 0;
        UnityEngine.Vector3 __0_inkIdVector__param = null /* "(0.00, 0.00, 0.00)" */;
        System.String penIdString = null;
        System.Single prevClickTime = 0.0f;
        System.Boolean __24__intnlparam = false;
        System.Boolean useDoubleClick = true;
        UnityEngine.Transform __3__intnlparam = null;
        System.Int32 inkColliderLayer = 0;
        System.Boolean useSurftraceMode = true;
        VRC.SDK3.Components.VRCPickup __0_get_pickup__ret = null;
        UnityEngine.Vector3[] __16__intnlparam = null;
        UnityEngine.GameObject __38__intnlparam = null;
        System.Boolean isUser = false;
        System.Boolean __0_get_isHeld__ret = false;
        UnityEngine.Vector2 mouseDelta = null /* "(0.00, 0.00)" */;
        System.Int32 __0_get_localPlayerId__ret = 0;
        System.Boolean __31__intnlparam = false;
        System.Boolean __0___0_TryGetLastLocalInk__ret = false;
        UnityEngine.Vector3 __const_UnityEngineVector3_0 = null /* "(1.00, 1.00, 1.00)" */;
        System.Boolean _isCheckedPointerRadius = false;
        System.Int32 __0_penId__param = 0;
        UnityEngine.Vector3 __0__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Component __7__intnlparam = null;
        System.Boolean _isCheckedLocalPlayerIdVector = false;
        System.String _logPrefix = null;
        System.Int32 __9__intnlparam = 0;
        UnityEngine.GameObject __23__intnlparam = null;
        System.Int32 __2_mode__param = 0;
        UnityEngine.LineRenderer __0_lineRenderer__param = null;
        UnityEngine.MeshCollider __0_get_inkPrefabCollider__ret = null;
        System.Int32 inkMeshLayer = 0;
        UnityEngine.Transform pointer = null;
        UnityEngine.Color __37__intnlparam = null /* "RGBA(0.000, 0.000, 0.000, 0.000)" */;
        UnityEngine.Vector3 center = null /* "(0.00, 0.00, 0.00)" */;
        System.Boolean __0___0__CheckId__ret = false;
        VRC.SDK3.Data.DataList localInkHistory = null /* [] */;
        System.Single[] __gintnl_SystemSingleArray_0 = null /* [0.0, 0.0, 0.0] */;
        System.Int32 __0_targetMode__param = 0;
        System.Int32 __15__intnlparam = 0;
        UnityEngine.MeshCollider _inkPrefabCollider = null;
        UnityEngine.Collider[] results32 = null /* [null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null, null,
                                                   null, null, null, null, null, null, null, null, null, null, null, null, null, null] */
            ;
        UnityEngine.Collider __0_target__param = null;
        UnityEngine.Vector3[] __4_data__param = null;
        System.Int32 __20__intnlparam = 0;
        System.Int32 __0_inkId__param = 0;
        System.Int32 __1_mode__param = 0;
        UnityEngine.Vector3 __4__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector3 _penIdVector_k__BackingField = null /* "(0.00, 0.00, 0.00)" */;
        System.Boolean isPickedUp = false;
        System.Boolean __3_value__param = false;
        UnityEngine.Vector3 __34__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        System.Single ratio = 0.0f;
        UnityEngine.Vector3 __0_idVector__param = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector3[] __12__intnlparam = null;
        VRC.Udon.UdonBehaviour __0_penManager__param = null;
        VRC.Udon.UdonBehaviour manager = null;
        UnityEngine.TrailRenderer trailRenderer = null;
        System.Int32 __29__intnlparam = 0;
        VRC.SDKBase.VRCPlayerApi __0_get_localPlayer__ret = null;
        System.Boolean isPointerEnabled = false;
        System.Single __0_get_pointerRadiusMultiplierForDesktop__ret = 0.0f;
        UnityEngine.Vector3 __2_inkIdVector__param = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector3 __26__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        System.Boolean __0_get_IsUser__ret = false;
        UnityEngine.Vector3 __const_UnityEngineVector3_2 = null /* "(0.00, 0.00, 1.00)" */;
        System.Boolean canBeErasedWithOtherPointers = true;
        System.String _respawnEventName = "Respawn";
        VRC.SDKBase.VRCPlayerApi + TrackingData headTracking = null /* "VRC.SDKBase.VRCPlayerApi+TrackingData" */;
        System.Int32 __1__intnlparam = 0;
        UnityEngine.Vector3[] __9_data__param = null;
        UnityEngine.Vector3 __33__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector3[] __18__intnlparam = null;
        UnityEngine.GameObject __0_ink__param = null;
        System.Single clickPosInterval = 0.01f;
        UnityEngine.Collider surftraceTarget = null;
        UnityEngine.Vector3 __const_UnityEngineVector3_1 = null /* "(0.00, 0.00, 0.00)" */;
        System.Int32 __11__intnlparam = 0;
        System.Single _pointerRadiusMultiplierForDesktop = 3.0f;
        System.Object[] __gintnl_SystemObjectArray_0 = null /* [null, null, null, null, null] */;
        System.Int32 penId = 0;
        System.Object __1_o__param = null;
        System.Object __0_o__param = null;
        UnityEngine.Color __0_c__param = null /* "RGBA(0.000, 0.000, 0.000, 0.000)" */;
        System.Object __2_o__param = null;
        UnityEngine.Transform inkPoolSynced = null;
        UnityEngine.Vector3 __1_ownerIdVector__param = null /* "(0.00, 0.00, 0.00)" */;
        VRC.SDK3.Components.VRCObjectSync __0_get_objectSync__ret = null;
        System.Boolean isScreenMode = false;
        VRC.SDK3.Components.VRCPickup _pickup = null;
        UnityEngine.Vector3[] __7_data__param = null;
        System.Single __0_get_pointerRadius__ret = 0.0f;
        UnityEngine.Vector3[] __30__intnlparam = null;
        UnityEngine.GameObject __25__intnlparam = null;
        UnityEngine.Vector3 __0_get_localPlayerIdVector__ret = null /* "(0.00, 0.00, 0.00)" */;
        System.Boolean __0__TakeOwnership__ret = false;
        System.Int32 __5__intnlparam = 0;
        UnityEngine.Material pointerMaterialNormal = null;
        System.Boolean __2_value__param = false;
        UnityEngine.Vector3[] __0___0__PackData__ret = null;
        System.Boolean __0_get_AllowCallPen__ret = false;
        UnityEngine.Vector3 __17__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        VRC.Udon.UdonBehaviour _alternativeObjectSync = null;
        UnityEngine.GameObject __39__intnlparam = null;
        UnityEngine.Vector2 _wh = null /* "(0.00, 0.00)" */;
        UnityEngine.Transform inkPoolRoot = null;
        System.Single scalar = 0.0f;
        UnityEngine.GameObject __22__intnlparam = null;
        System.Int32 inkColliderLayerMask = 0;
        UnityEngine.Color logColor = null /* "RGBA(0.949, 0.490, 0.290, 1.000)" */;
        System.String __36__intnlparam = null;
        UnityEngine.MaterialPropertyBlock propertyBlock = null;
        UnityEngine.Canvas screenOverlay = null;
        System.Boolean __1_value__param = false;
        UnityEngine.Vector3 __14__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        System.Boolean isRoundedTrailShader = false;
        VRC.Udon.UdonBehaviour penManager = null;
        System.Boolean _isUserInVR = false;
        UnityEngine.Quaternion __const_UnityEngineQuaternion_0 = null /* "(0.00000, 0.00000, 0.00000, 1.00000)" */;
        System.Boolean __0_get_isSurftraceMode__ret = false;
        UnityEngine.Vector3[] __8_data__param = null;
        UnityEngine.Vector3 __0_get_penIdVector__ret = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector2 clampWH = null /* "(0.00, 0.00)" */;
        VRC.SDKBase.VRCPlayerApi _localPlayer = null;
        System.Int32 currentSyncState = 0;
        UnityEngine.Vector3 __1_inkIdVector__param = null /* "(0.00, 0.00, 0.00)" */;
        System.Boolean _isCheckedIsUserInVR = false;
        UnityEngine.Transform __2__intnlparam = null;
        System.Single sensitivity = 0.75f;
        UnityEngine.LineRenderer __1_lineRenderer__param = null;
        UnityEngine.Vector3 __0_penIdVector__param = null /* "(0.00, 0.00, 0.00)" */;
        VRC.SDK3.Components.VRCObjectSync _objectSync = null;
        UnityEngine.Vector3 __28__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Collider[] results4 = null /* [null, null, null, null] */;
        UnityEngine.Transform inkPoolNotSynced = null;
        UnityEngine.Vector3 headPos = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Quaternion headRot = null /* "(0.00000, 0.00000, 0.00000, 0.00000)" */;
        UnityEngine.Vector2 wh = null /* "(0.00, 0.00)" */;
        System.Boolean __0_get_isUserInVR__ret = false;
        System.Int32 surftraceMask = -1;
        UnityEngine.Renderer marker = null;
        UnityEngine.TrailRenderer __0_trailRenderer__param = null;
        UnityEngine.Vector3 __21__intnlparam = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Vector2 __const_UnityEngineVector2_0 = null /* "(0.00, 0.00)" */;
        UnityEngine.Transform inkPool = null;
        UnityEngine.Vector3[] __6_data__param = null;
        System.Single inkWidth = 0.0f;
        UnityEngine.Vector3 _localPlayerIdVector = null /* "(0.00, 0.00, 0.00)" */;
        UnityEngine.Transform inkPositionChild = null;
        System.Int32 __2_inkId__param = 0;

        public void get_AllowCallPen()
        {
            __0_get_AllowCallPen__ret = allowCallPen;
            return;
        }

        void get_pointerRadius()
        {
            UnityEngine.Vector3 __lcl_s_UnityEngineVector3_0;
            UnityEngine.SphereCollider __lcl_sphereCollider_UnityEngineSphereCollider_0;
            if (_isCheckedPointerRadius)
            {
                __0_get_pointerRadius__ret = _pointerRadius;
                return;
            }
            else
            {
                __lcl_sphereCollider_UnityEngineSphereCollider_0 = pointer.transform.GetComponent(
                    null /* "UnityEngine.SphereCollider, UnityEngine.PhysicsModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
                __lcl_sphereCollider_UnityEngineSphereCollider_0.enabled = false;
                __lcl_s_UnityEngineVector3_0 = pointer.lossyScale;
                __gintnl_SystemSingleArray_0.Set(0, __lcl_s_UnityEngineVector3_0.x);
                __gintnl_SystemSingleArray_0.Set(1, __lcl_s_UnityEngineVector3_0.y);
                __gintnl_SystemSingleArray_0.Set(2, __lcl_s_UnityEngineVector3_0.z);
                _pointerRadius =
                    UnityEngine.Mathf.Max(0.01f, UnityEngine.Mathf.Min(__gintnl_SystemSingleArray_0)) * __lcl_sphereCollider_UnityEngineSphereCollider_0.radius;
                _isCheckedPointerRadius = true;
                __0_get_pointerRadius__ret = _pointerRadius;
                return;
            }
        }

        void get_pointerRadiusMultiplierForDesktop()
        {
            get_isUserInVR();
            if (__0_get_isUserInVR__ret)
            {
                __0_get_pointerRadiusMultiplierForDesktop__ret = 1.0f;
            }
            else
            {
                __0_get_pointerRadiusMultiplierForDesktop__ret = UnityEngine.Mathf.Abs(_pointerRadiusMultiplierForDesktop);
            }
            return;
        }

        void get_inkPrefabCollider()
        {
            if (VRC.SDKBase.Utilities.IsValid(_inkPrefabCollider))
            {
                __0_get_inkPrefabCollider__ret = _inkPrefabCollider;
            }
            else
            {
                _inkPrefabCollider = inkPrefab.transform.GetComponentInChildren(
                    true, null /* "UnityEngine.MeshCollider, UnityEngine.PhysicsModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
                __0_get_inkPrefabCollider__ret = _inkPrefabCollider;
            }
            return;
        }

        public void get_IsUser()
        {
            __0_get_IsUser__ret = isUser;
            return;
        }

        void get_pickup()
        {
            if (VRC.SDKBase.Utilities.IsValid(_pickup))
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
            if (VRC.SDKBase.Utilities.IsValid(_objectSync))
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

        public void get_penIdVector()
        {
            __0_get_penIdVector__ret = _penIdVector_k__BackingField;
            return;
        }

        void function_0()
        {
            _penIdVector_k__BackingField = __0_value__param;
            return;
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
            if (!_isCheckedLocalPlayerId)
            {
                get_localPlayer();
                _isCheckedLocalPlayerId = VRC.SDKBase.Utilities.IsValid(__0_get_localPlayer__ret);
                if (_isCheckedLocalPlayerId)
                {
                    get_localPlayer();
                    _localPlayerId = __0_get_localPlayer__ret.playerId;
                    __0_get_localPlayerId__ret = _localPlayerId;
                }
                else
                {
                    __0_get_localPlayerId__ret = 0;
                }
            }
            else
            {
                __0_get_localPlayerId__ret = _localPlayerId;
            }
            return;
        }

        void get_localPlayerIdVector()
        {
            if (_isCheckedLocalPlayerIdVector)
            {
                __0_get_localPlayerIdVector__ret = _localPlayerIdVector;
                return;
            }
            else
            {
                get_localPlayerId();
                __1__intnlparam = __0_get_localPlayerId__ret;
                function_27();
                _localPlayerIdVector = __0__intnlparam;
                _isCheckedLocalPlayerIdVector = true;
                __0_get_localPlayerIdVector__ret = _localPlayerIdVector;
                return;
            }
        }

        void get_isUserInVR()
        {
            System.Boolean __intnl_SystemBoolean_4;
            if (!_isCheckedIsUserInVR)
            {
                get_localPlayer();
                _isCheckedIsUserInVR = VRC.SDKBase.Utilities.IsValid(__0_get_localPlayer__ret);
                __intnl_SystemBoolean_4 = _isCheckedIsUserInVR;
                if (__intnl_SystemBoolean_4)
                {
                    get_localPlayer();
                    _isUserInVR = __0_get_localPlayer__ret.IsUserInVR();
                    __intnl_SystemBoolean_4 = _isUserInVR;
                }
                __0_get_isUserInVR__ret = __intnl_SystemBoolean_4;
            }
            else
            {
                __0_get_isUserInVR__ret = _isUserInVR;
            }
            return;
        }

        public void __0__Init()
        {
            UnityEngine.Transform __intnl_UnityEngineTransform_2;
            UnityEngine.GameObject __intnl_UnityEngineGameObject_0;
            System.String __lcl_unique_SystemString_0;
            UnityEngine.GameObject __lcl_inkPoolRootGO_UnityEngineGameObject_0;
            penManager = __0_penManager__param;
            _UpdateInkData();
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
                __intnl_UnityEngineTransform_2 = null;
                __2__intnlparam = inkPoolRoot;
                __3__intnlparam = __intnl_UnityEngineTransform_2;
                function_28();
                inkPoolRoot.SetAsFirstSibling();
                __intnl_UnityEngineGameObject_0 = inkPoolRoot.gameObject;
                __intnl_UnityEngineGameObject_0.SetActive(true);
            }
            inkPool = syncer.transform;
            __2__intnlparam = inkPool;
            __3__intnlparam = inkPoolRoot;
            function_28();
            __lcl_unique_SystemString_0 = VRC.SDKBase.Networking.GetUniqueName(this.gameObject);
            if (System.String.IsNullOrEmpty(__lcl_unique_SystemString_0))
            {
                penId = 0;
            }
            else
            {
                penId = __lcl_unique_SystemString_0.GetHashCode();
            }
            __5__intnlparam = penId;
            function_29();
            __0_value__param = __4__intnlparam;
            function_0();
            get_penIdVector();
            get_penIdVector();
            get_penIdVector();
            penIdString =
                System.String.Format("0x{0:x2}{1:x3}{2:x3}", System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__0_get_penIdVector__ret.x))),
                                     System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__0_get_penIdVector__ret.y))),
                                     System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__0_get_penIdVector__ret.z))));
            inkPool.name = System.String.Format("{0} ({1})", "InkPool", penIdString);
            __0_penManager__param.SendCustomEvent("get_AllowCallPen");
            allowCallPen = __0_penManager__param.GetProgramVariable("__0_get_AllowCallPen__ret");
            __intnl_UnityEngineTransform_2 = inkPoolRoot.transform;
            __7__intnlparam = __intnl_UnityEngineTransform_2;
            function_30();
            manager = __6__intnlparam;
            manager.SetProgramVariable("__0_penId__param", penId);
            manager.SetProgramVariable("__0_pen__param", this);
            manager.SendCustomEvent("__0_Register");
            syncer.SetProgramVariable("__0_pen__param", this);
            syncer.SendCustomEvent("__0__RegisterPen");
            syncer.SendCustomEvent("get_InkPoolSynced");
            inkPoolSynced = syncer.GetProgramVariable("__0_get_InkPoolSynced__ret");
            syncer.SendCustomEvent("get_InkPoolNotSynced");
            inkPoolNotSynced = syncer.GetProgramVariable("__0_get_InkPoolNotSynced__ret");
            get_pickup();
            __0_get_pickup__ret.InteractionText = "QvPen";
            get_pickup();
            __0_get_pickup__ret.UseText = "Draw";
            pointerRenderer = pointer.transform.GetComponent(
                null /* "UnityEngine.Renderer, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
            __intnl_UnityEngineGameObject_0 = pointer.gameObject;
            __intnl_UnityEngineGameObject_0.SetActive(false);
            get_pointerRadiusMultiplierForDesktop();
            pointer.transform.localScale = pointer.transform.localScale * __0_get_pointerRadiusMultiplierForDesktop__ret;
            marker.transform.localScale = __const_UnityEngineVector3_0 * inkWidth;
            get_isUserInVR();
            if (__0_get_isUserInVR__ret)
            {
                clickPosInterval = 0.005f;
            }
            else
            {
                clickPosInterval = 0.001f;
            }
            return;
        }

        public void _UpdateInkData()
        {
            UnityEngine.Shader __lcl_shader_UnityEngineShader_0;
            System.Object __intnl_SystemObject_7;
            UnityEngine.Material __lcl_material_UnityEngineMaterial_0;
            inkWidth = System.Convert.ToSingle(penManager.GetProgramVariable("inkWidth"));
            inkMeshLayer = System.Convert.ToInt32(penManager.GetProgramVariable("inkMeshLayer"));
            inkColliderLayer = System.Convert.ToInt32(penManager.GetProgramVariable("inkColliderLayer"));
            inkColliderLayerMask = 1 << inkColliderLayer;
            inkPrefab.gameObject.layer = inkMeshLayer;
            trailRenderer.gameObject.layer = inkMeshLayer;
            get_inkPrefabCollider();
            __0_get_inkPrefabCollider__ret.gameObject.layer = inkColliderLayer;
            __lcl_material_UnityEngineMaterial_0 = penManager.GetProgramVariable("pcInkMaterial");
            inkPrefab.material = __lcl_material_UnityEngineMaterial_0;
            trailRenderer.material = __lcl_material_UnityEngineMaterial_0;
            if (VRC.SDKBase.Utilities.IsValid(__lcl_material_UnityEngineMaterial_0))
            {
                __lcl_shader_UnityEngineShader_0 = __lcl_material_UnityEngineMaterial_0.shader;
                if (VRC.SDKBase.Utilities.IsValid(__lcl_shader_UnityEngineShader_0))
                {
                    penManager.SendCustomEvent("get_roundedTrailShader");
                    __intnl_SystemObject_7 = penManager.GetProgramVariable("__0_get_roundedTrailShader__ret");
                    isRoundedTrailShader = __lcl_shader_UnityEngineShader_0 == __intnl_SystemObject_7;
                    isRoundedTrailShader = isRoundedTrailShader | __lcl_shader_UnityEngineShader_0.name.Contains("rounded_trail");
                }
            }
            if (isRoundedTrailShader)
            {
                inkPrefab.widthMultiplier = 0.0f;
                propertyBlock = new UnityEngine.MaterialPropertyBlock();
                inkPrefab.GetPropertyBlock(propertyBlock);
                propertyBlock.SetFloat("_Width", inkWidth);
                inkPrefab.SetPropertyBlock(propertyBlock);
                trailRenderer.widthMultiplier = 0.0f;
                propertyBlock.Clear();
                trailRenderer.GetPropertyBlock(propertyBlock);
                propertyBlock.SetFloat("_Width", inkWidth);
                trailRenderer.SetPropertyBlock(propertyBlock);
            }
            else
            {
                inkPrefab.widthMultiplier = inkWidth;
                trailRenderer.widthMultiplier = inkWidth;
            }
            __intnl_SystemObject_7 = penManager.GetProgramVariable("colorGradient");
            inkPrefab.colorGradient = __intnl_SystemObject_7;
            trailRenderer.colorGradient = penManager.GetProgramVariable("colorGradient");
            surftraceMask = (UnityEngine.LayerMask)penManager.GetProgramVariable("surftraceMask");
            return;
        }

        public void __0__CheckId()
        {
            UnityEngine.Vector3 __intnl_UnityEngineVector3_3;
            __intnl_UnityEngineVector3_3 = __0_idVector__param;
            get_penIdVector();
            __0___0__CheckId__ret = __intnl_UnityEngineVector3_3 == __0_get_penIdVector__ret;
            return;
        }

        public void _update()
        {
            System.Boolean __intnl_SystemBoolean_11;
            System.String __intnl_SystemString_3;
            System.Single __intnl_SystemSingle_9;
            get_isUserInVR();
            __intnl_SystemBoolean_11 = __0_get_isUserInVR__ret;
            if (!__intnl_SystemBoolean_11)
            {
                __intnl_SystemBoolean_11 = !isUser;
            }
            if (__intnl_SystemBoolean_11)
            {
                return;
            }
            else
            {
                if (UnityEngine.Input.GetKeyUp(null /* 9 */))
                {
                    function_2();
                }
                if (UnityEngine.Input.anyKey)
                {
                    if (!UnityEngine.Input.GetKeyDown(null /* 8 */))
                    {
                        if (!UnityEngine.Input.GetKeyDown(null /* 9 */))
                        {
                            if (UnityEngine.Input.GetKey(null /* 9 */))
                            {
                                if (!UnityEngine.Input.GetKeyDown(null /* 127 */))
                                {
                                    if (!UnityEngine.Input.GetKey(null /* 278 */))
                                    {
                                        if (!UnityEngine.Input.GetKey(null /* 273 */))
                                        {
                                            if (UnityEngine.Input.GetKey(null /* 274 */))
                                            {
                                                __intnl_SystemSingle_9 = sensitivity - 0.001f;
                                                sensitivity = UnityEngine.Mathf.Max(__intnl_SystemSingle_9, 0.01f);
                                                __intnl_SystemString_3 = System.String.Format("Sensitivity -> {0:f3}", sensitivity);
                                                __0_o__param = __intnl_SystemString_3;
                                                function_23();
                                            }
                                        }
                                        else
                                        {
                                            __intnl_SystemSingle_9 = sensitivity + 0.001f;
                                            sensitivity = UnityEngine.Mathf.Min(__intnl_SystemSingle_9, 5.0f);
                                            __intnl_SystemString_3 = System.String.Format("Sensitivity -> {0:f3}", sensitivity);
                                            __0_o__param = __intnl_SystemString_3;
                                            function_23();
                                        }
                                    }
                                    else
                                    {
                                        penManager.SendCustomNetworkEvent(null /* 0 */, "Respawn");
                                    }
                                }
                                else
                                {
                                    penManager.SendCustomNetworkEvent(null /* 0 */, "Clear");
                                }
                            }
                        }
                        else
                        {
                            function_1();
                        }
                    }
                    else
                    {
                        _UndoDraw();
                    }
                    return;
                }
                else
                {
                    return;
                }
            }
        }

        void function_1()
        {
            isScreenMode = true;
            marker.enabled = true;
            _wh = __const_UnityEngineVector2_0;
            screenOverlay.gameObject.SetActive(true);
            wh = screenOverlay.transform
                     .GetComponent(null /* "UnityEngine.RectTransform, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */)
                     .rect.size;
            screenOverlay.gameObject.SetActive(false);
            clampWH = wh / 3763.2002f;
            ratio = 2160.0f / wh.y;
            return;
        }

        void function_2()
        {
            isScreenMode = false;
            function_3();
            if (!__0_get_isSurftraceMode__ret)
            {
                marker.enabled = false;
            }
            this.SendCustomNetworkEvent(null /* 0 */, "ChangeStateToPenIdle");
            inkPositionChild.SetLocalPositionAndRotation(__const_UnityEngineVector3_1, __const_UnityEngineQuaternion_0);
            trailRenderer.transform.SetPositionAndRotation(inkPositionChild.position, inkPositionChild.rotation);
            return;
        }

        public void _lateUpdate()
        {
            UnityEngine.Vector3 __lcl_closestPoint_UnityEngineVector3_0;
            UnityEngine.Vector3 __intnl_UnityEngineVector3_6;
            System.Single __lcl_deltaDistance_SystemSingle_0;
            System.Single __intnl_SystemSingle_11;
            System.Single __intnl_SystemSingle_13;
            System.Single __intnl_SystemSingle_14;
            UnityEngine.Vector3 __intnl_UnityEngineVector3_5;
            System.Boolean __intnl_SystemBoolean_21;
            UnityEngine.Transform __intnl_UnityEngineTransform_10;
            UnityEngine.Vector2 __intnl_UnityEngineVector2_1;
            UnityEngine.Vector3 __intnl_UnityEngineVector3_7;
            System.Single __lcl_distance_SystemSingle_0;
            System.Boolean __intnl_SystemBoolean_22;
            UnityEngine.Vector3 __lcl_inkPositionPosition_UnityEngineVector3_0;
            UnityEngine.Vector2 __intnl_UnityEngineVector2_0;
            UnityEngine.Quaternion __intnl_UnityEngineQuaternion_1;
            get_isHeld();
            if (__0_get_isHeld__ret)
            {
                get_isUserInVR();
                __intnl_SystemBoolean_22 = !__0_get_isUserInVR__ret;
                if (__intnl_SystemBoolean_22)
                {
                    __intnl_SystemBoolean_22 = isUser;
                }
                __intnl_SystemBoolean_21 = __intnl_SystemBoolean_22;
                if (__intnl_SystemBoolean_21)
                {
                    __intnl_SystemBoolean_21 = UnityEngine.Input.GetKey(null /* 9 */);
                }
                if (__intnl_SystemBoolean_21)
                {
                    get_localPlayer();
                    headTracking = __0_get_localPlayer__ret.GetTrackingData(null /* 0 */);
                    headPos = headTracking.position;
                    headRot = headTracking.rotation;
                    __intnl_UnityEngineVector3_5 = headRot * __const_UnityEngineVector3_2;
                    __intnl_UnityEngineVector3_6 = headRot * __const_UnityEngineVector3_2;
                    __intnl_UnityEngineVector3_7 = this.transform.position;
                    __intnl_SystemSingle_11 = UnityEngine.Vector3.Dot(__intnl_UnityEngineVector3_6, __intnl_UnityEngineVector3_7 - headPos);
                    center = __intnl_UnityEngineVector3_5 * __intnl_SystemSingle_11;
                    scalar = ratio * UnityEngine.Vector3.Dot(headRot * __const_UnityEngineVector3_2, center);
                    center = center + headPos;
                    __intnl_SystemSingle_13 = UnityEngine.Input.GetAxis("Mouse X");
                    mouseDelta.x = __intnl_SystemSingle_13;
                    __intnl_UnityEngineVector2_0 = mouseDelta;
                    __intnl_SystemSingle_14 = UnityEngine.Input.GetAxis("Mouse Y");
                    mouseDelta.y = __intnl_SystemSingle_14;
                    __intnl_UnityEngineVector2_1 = mouseDelta;
                    __intnl_SystemSingle_13 = UnityEngine.Time.deltaTime;
                    __intnl_SystemSingle_14 = sensitivity * __intnl_SystemSingle_13;
                    __intnl_UnityEngineVector2_0 = __intnl_SystemSingle_14 * mouseDelta;
                    _wh = _wh + __intnl_UnityEngineVector2_0;
                    __intnl_UnityEngineVector2_1 = !clampWH;
                    _wh = UnityEngine.Vector2.Min(UnityEngine.Vector2.Max(_wh, __intnl_UnityEngineVector2_1), clampWH);
                    inkPositionChild.SetPositionAndRotation(center + headRot * (UnityEngine.Vector2)_wh * scalar, headRot);
                }
                function_3();
                if (__0_get_isSurftraceMode__ret)
                {
                    if (isScreenMode)
                    {
                        __lcl_inkPositionPosition_UnityEngineVector3_0 = inkPositionChild.position;
                    }
                    else
                    {
                        __lcl_inkPositionPosition_UnityEngineVector3_0 = inkPosition.position;
                    }
                    __lcl_closestPoint_UnityEngineVector3_0 = surftraceTarget.ClosestPoint(__lcl_inkPositionPosition_UnityEngineVector3_0);
                    __lcl_distance_SystemSingle_0 =
                        UnityEngine.Vector3.Distance(__lcl_closestPoint_UnityEngineVector3_0, __lcl_inkPositionPosition_UnityEngineVector3_0);
                    __intnl_SystemSingle_11 = inkWidth / 1.999f;
                    __intnl_UnityEngineVector3_5 = UnityEngine.Vector3.MoveTowards(__lcl_closestPoint_UnityEngineVector3_0,
                                                                                   __lcl_inkPositionPosition_UnityEngineVector3_0, __intnl_SystemSingle_11);
                    inkPositionChild.position = __intnl_UnityEngineVector3_5;
                    if (__lcl_distance_SystemSingle_0 > 1.0f)
                    {
                        function_5();
                    }
                }
                if (!isPointerEnabled)
                {
                    if (isUser)
                    {
                        __intnl_SystemSingle_11 = UnityEngine.Time.deltaTime;
                        __lcl_deltaDistance_SystemSingle_0 = __intnl_SystemSingle_11 * 32.0f;
                        __intnl_UnityEngineTransform_10 = trailRenderer.transform;
                        __intnl_UnityEngineVector3_5 = trailRenderer.transform.position;
                        __intnl_UnityEngineVector3_6 = inkPositionChild.position;
                        __intnl_UnityEngineVector3_7 =
                            UnityEngine.Vector3.Lerp(__intnl_UnityEngineVector3_5, __intnl_UnityEngineVector3_6, __lcl_deltaDistance_SystemSingle_0);
                        __intnl_UnityEngineQuaternion_1 = trailRenderer.transform.rotation;
                        __intnl_UnityEngineTransform_10.SetPositionAndRotation(
                            __intnl_UnityEngineVector3_7,
                            UnityEngine.Quaternion.Lerp(__intnl_UnityEngineQuaternion_1, inkPositionChild.rotation, __lcl_deltaDistance_SystemSingle_0));
                    }
                    else
                    {
                        __intnl_UnityEngineTransform_10 = trailRenderer.transform;
                        __intnl_UnityEngineVector3_5 = inkPositionChild.position;
                        __intnl_UnityEngineQuaternion_1 = inkPositionChild.rotation;
                        __intnl_UnityEngineTransform_10.SetPositionAndRotation(__intnl_UnityEngineVector3_5, __intnl_UnityEngineQuaternion_1);
                    }
                }
                return;
            }
            else
            {
                return;
            }
        }

        public void _postLateUpdate()
        {
            System.Boolean __intnl_SystemBoolean_26;
            UnityEngine.LineRenderer __lcl_lineRenderer_UnityEngineLineRenderer_0;
            UnityEngine.Transform __lcl_t2_UnityEngineTransform_0;
            System.Int32 __lcl_count_SystemInt32_0;
            System.Boolean __intnl_SystemBoolean_29;
            UnityEngine.Transform __lcl_t1_UnityEngineTransform_0;
            System.Boolean __intnl_SystemBoolean_27;
            System.Int32 __lcl_i_SystemInt32_0;
            System.Boolean __intnl_SystemBoolean_25;
            UnityEngine.Collider __lcl_other_UnityEngineCollider_0;
            UnityEngine.Transform __lcl_t3_UnityEngineTransform_0;
            System.Boolean __intnl_SystemBoolean_28;
            if (isUser)
            {
                if (isPointerEnabled)
                {
                    get_pointerRadius();
                    __lcl_count_SystemInt32_0 =
                        UnityEngine.Physics.OverlapSphereNonAlloc(pointer.position, __0_get_pointerRadius__ret, results4, inkColliderLayerMask, null /* 1 */);
                    __lcl_i_SystemInt32_0 = 0;
                    if (__lcl_i_SystemInt32_0 < __lcl_count_SystemInt32_0)
                    {
                        __lcl_other_UnityEngineCollider_0 = results4.Get(__lcl_i_SystemInt32_0);
                        __intnl_SystemBoolean_26 = VRC.SDKBase.Utilities.IsValid(__lcl_other_UnityEngineCollider_0);
                        if (__intnl_SystemBoolean_26)
                        {
                            __lcl_t1_UnityEngineTransform_0 = __lcl_other_UnityEngineCollider_0.transform.parent;
                            __intnl_SystemBoolean_26 = VRC.SDKBase.Utilities.IsValid(__lcl_t1_UnityEngineTransform_0);
                        }
                        __intnl_SystemBoolean_25 = __intnl_SystemBoolean_26;
                        if (__intnl_SystemBoolean_25)
                        {
                            __lcl_t2_UnityEngineTransform_0 = __lcl_t1_UnityEngineTransform_0.parent;
                            __intnl_SystemBoolean_25 = VRC.SDKBase.Utilities.IsValid(__lcl_t2_UnityEngineTransform_0);
                        }
                        if (__intnl_SystemBoolean_25)
                        {
                            if (canBeErasedWithOtherPointers)
                            {
                                __lcl_t3_UnityEngineTransform_0 = __lcl_t2_UnityEngineTransform_0.parent;
                                __intnl_SystemBoolean_28 = VRC.SDKBase.Utilities.IsValid(__lcl_t3_UnityEngineTransform_0);
                                if (__intnl_SystemBoolean_28)
                                {
                                    __intnl_SystemBoolean_28 = __lcl_t3_UnityEngineTransform_0.parent == inkPoolRoot;
                                }
                                __intnl_SystemBoolean_27 = __intnl_SystemBoolean_28;
                            }
                            else
                            {
                                __intnl_SystemBoolean_27 = __lcl_t2_UnityEngineTransform_0.parent == inkPool;
                            }
                            if (__intnl_SystemBoolean_27)
                            {
                                __lcl_lineRenderer_UnityEngineLineRenderer_0 = __lcl_other_UnityEngineCollider_0.transform.GetComponentInParent(
                                    null /* "UnityEngine.LineRenderer, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
                                __intnl_SystemBoolean_29 = VRC.SDKBase.Utilities.IsValid(__lcl_lineRenderer_UnityEngineLineRenderer_0);
                                if (__intnl_SystemBoolean_29)
                                {
                                    __intnl_SystemBoolean_29 = __lcl_lineRenderer_UnityEngineLineRenderer_0.positionCount > 0;
                                }
                                if (__intnl_SystemBoolean_29)
                                {
                                    __0_ink__param = __lcl_lineRenderer_UnityEngineLineRenderer_0.gameObject;
                                    function_17();
                                }
                            }
                        }
                        results4.Set(__lcl_i_SystemInt32_0, null);
                        __lcl_i_SystemInt32_0 = __lcl_i_SystemInt32_0 + 1;
                        goto label_bb_0000256c;
                    }
                    else
                    {
                        goto label_bb_0000290c;
                    }
                }
            label_bb_0000290c:
                return;
            }
            else
            {
                return;
            }
        label_bb_0000256c:
        }

        void function_3()
        {
            __0_get_isSurftraceMode__ret = (UnityEngine.Object)surftraceTarget;
            return;
        }

        public void _onTriggerEnter()
        {
            System.Boolean __intnl_SystemBoolean_31;
            System.Single __lcl_distance_SystemSingle_1;
            System.Boolean __intnl_SystemBoolean_32;
            System.Boolean __intnl_SystemBoolean_35;
            System.Boolean __intnl_SystemBoolean_30;
            __intnl_SystemBoolean_32 = isUser;
            if (__intnl_SystemBoolean_32)
            {
                __intnl_SystemBoolean_32 = useSurftraceMode;
            }
            __intnl_SystemBoolean_31 = __intnl_SystemBoolean_32;
            if (__intnl_SystemBoolean_31)
            {
                __intnl_SystemBoolean_31 = VRC.SDKBase.Utilities.IsValid(onTriggerEnterOther);
            }
            __intnl_SystemBoolean_30 = __intnl_SystemBoolean_31;
            if (__intnl_SystemBoolean_30)
            {
                __intnl_SystemBoolean_30 = !onTriggerEnterOther.isTrigger;
            }
            if (__intnl_SystemBoolean_30)
            {
                if ((1 << onTriggerEnterOther.gameObject.layer & surftraceMask) == 0)
                {
                    return;
                }
                else
                {
                    __intnl_SystemBoolean_35 =
                        onTriggerEnterOther.GetType() ==
                        null /* "UnityEngine.MeshCollider, UnityEngine.PhysicsModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */;
                    if (__intnl_SystemBoolean_35)
                    {
                        __intnl_SystemBoolean_35 = !onTriggerEnterOther.convex;
                    }
                    if (__intnl_SystemBoolean_35)
                    {
                        return;
                    }
                    else
                    {
                        __lcl_distance_SystemSingle_1 =
                            UnityEngine.Vector3.Distance(onTriggerEnterOther.ClosestPoint(inkPosition.position), inkPosition.position);
                        if (__lcl_distance_SystemSingle_1 < 0.05f)
                        {
                            __0_target__param = onTriggerEnterOther;
                            function_4();
                        }
                        return;
                    }
                }
            }
            else
            {
                goto label_bb_00002c5c;
            }
        label_bb_00002c5c:
        }

        void function_4()
        {
            surftraceTarget = __0_target__param;
            marker.enabled = true;
            return;
        }

        void function_5()
        {
            surftraceTarget = null;
            if (!isScreenMode)
            {
                marker.enabled = false;
            }
            this.SendCustomNetworkEvent(null /* 0 */, "ChangeStateToPenIdle");
            inkPositionChild.SetLocalPositionAndRotation(__const_UnityEngineVector3_1, __const_UnityEngineQuaternion_0);
            trailRenderer.transform.SetPositionAndRotation(inkPositionChild.position, inkPositionChild.rotation);
            return;
        }

        public void _onPickup()
        {
            isUser = true;
            manager.SetProgramVariable("__1_pen__param", this);
            manager.SendCustomEvent("__0_SetLastUsedPen");
            penManager.SendCustomEvent("OnPenPickup");
            penManager.SendCustomEvent("_TakeOwnership");
            penManager.SendCustomNetworkEvent(null /* 0 */, "StartUsing");
            this.SendCustomNetworkEvent(null /* 0 */, "ChangeStateToPenIdle");
            return;
        }

        public void _onDrop()
        {
            isUser = false;
            penManager.SendCustomEvent("OnPenDrop");
            penManager.SendCustomNetworkEvent(null /* 0 */, "EndUsing");
            this.SendCustomNetworkEvent(null /* 0 */, "ChangeStateToPenIdle");
            penManager.SendCustomEvent("_ClearSyncBuffer");
            function_2();
            function_5();
            return;
        }

        public void _onPickupUseDown()
        {
            System.Boolean __intnl_SystemBoolean_41;
            System.Boolean __intnl_SystemBoolean_39;
            System.Boolean __intnl_SystemBoolean_42;
            System.Boolean __intnl_SystemBoolean_40;
            System.String __intnl_SystemString_4;
            __intnl_SystemBoolean_40 = useDoubleClick;
            if (__intnl_SystemBoolean_40)
            {
                __intnl_SystemBoolean_40 = UnityEngine.Time.time - prevClickTime < 0.2f;
            }
            __intnl_SystemBoolean_39 = __intnl_SystemBoolean_40;
            if (__intnl_SystemBoolean_39)
            {
                __intnl_SystemBoolean_39 = UnityEngine.Vector3.Distance(inkPosition.position, prevClickPos) < clickPosInterval;
            }
            if (__intnl_SystemBoolean_39)
            {
                prevClickTime = 0.0f;
                __intnl_SystemBoolean_41 = currentState == 0;
                if (__intnl_SystemBoolean_41)
                {
                    __intnl_SystemBoolean_42 = UnityEngine.Vector3.Distance(inkPosition.position, prevClickPos) > 0.0f;
                    if (__intnl_SystemBoolean_42)
                    {
                        _UndoDraw();
                    }
                    this.SendCustomNetworkEvent(null /* 0 */, "ChangeStateToEraseIdle");
                }
                else if (currentState == 2)
                {
                    this.SendCustomNetworkEvent(null /* 0 */, "ChangeStateToPenIdle");
                }
                else
                {
                    __9__intnlparam = currentState;
                    function_31();
                    __intnl_SystemString_4 = System.String.Format("Unexpected state : {0} at {1} Double Clicked", __8__intnlparam, "OnPickupUseDown");
                    __2_o__param = __intnl_SystemString_4;
                    function_25();
                }
            }
            else
            {
                prevClickTime = UnityEngine.Time.time;
                prevClickPos = inkPosition.position;
                __intnl_SystemBoolean_41 = currentState == 0;
                if (!__intnl_SystemBoolean_41)
                {
                    __intnl_SystemBoolean_42 = currentState == 2;
                    if (__intnl_SystemBoolean_42)
                    {
                        this.SendCustomNetworkEvent(null /* 0 */, "ChangeStateToEraseUsing");
                        function_22();
                    }
                    else
                    {
                        __9__intnlparam = currentState;
                        function_31();
                        __intnl_SystemString_4 = System.String.Format("Unexpected state : {0} at {1}", __8__intnlparam, "OnPickupUseDown");
                        __2_o__param = __intnl_SystemString_4;
                        function_25();
                    }
                }
                else
                {
                    this.SendCustomNetworkEvent(null /* 0 */, "ChangeStateToPenUsing");
                }
            }
            return;
        }

        public void _onPickupUseUp()
        {
            switch (currentState)
            {
                case 0:
                    __9__intnlparam = currentState;
                    function_31();
                    __0_o__param = System.String.Format("Change state : {0} to {1}", "EraserIdle", __8__intnlparam);
                    function_23();
                    break;
                case 1:
                    this.SendCustomNetworkEvent(null /* 0 */, "ChangeStateToPenIdle");
                    break;
                case 2:
                    __9__intnlparam = currentState;
                    function_31();
                    __0_o__param = System.String.Format("Change state : {0} to {1}", "PenIdle", __8__intnlparam);
                    function_23();
                    break;
                case 3:
                    this.SendCustomNetworkEvent(null /* 0 */, "ChangeStateToEraseIdle");
                    break;
                default:
                    __9__intnlparam = currentState;
                    function_31();
                    __2_o__param = System.String.Format("Unexpected state : {0} at {1}", __8__intnlparam, "OnPickupUseUp");
                    function_25();
                    break;
            }
            return;
        }

        public void __0__SetUseDoubleClick()
        {
            useDoubleClick = __1_value__param;
            if (isUser)
            {
                this.SendCustomNetworkEvent(null /* 0 */, "ChangeStateToPenIdle");
            }
            return;
        }

        public void __0__SetEnabledLateSync()
        {
            enabledLateSync = __2_value__param;
            return;
        }

        public void __0__SetUseSurftraceMode()
        {
            useSurftraceMode = __3_value__param;
            if (isUser)
            {
                this.SendCustomNetworkEvent(null /* 0 */, "ChangeStateToPenIdle");
            }
            return;
        }

        public void _onEnable()
        {
            if (VRC.SDKBase.Utilities.IsValid(inkPool))
            {
                inkPool.gameObject.SetActive(true);
            }
            return;
        }

        public void _onDisable()
        {
            if (VRC.SDKBase.Utilities.IsValid(inkPool))
            {
                inkPool.gameObject.SetActive(false);
            }
            return;
        }

        public void _onDestroy()
        {
            _Clear();
            if (VRC.SDKBase.Utilities.IsValid(inkPool))
            {
                UnityEngine.Object.Destroy(inkPool.gameObject);
            }
            return;
        }

        public void ChangeStateToPenIdle()
        {
            switch (currentState)
            {
                case 1:
                    function_7();
                    break;
                case 2:
                    function_10();
                    break;
                case 3:
                    function_9();
                    function_10();
                    break;
                default:
                    break;
            }
            currentState = 0;
            return;
        }

        public void ChangeStateToPenUsing()
        {
            switch (currentState)
            {
                case 0:
                    function_6();
                    break;
                case 2:
                    function_10();
                    function_6();
                    break;
                case 3:
                    function_9();
                    function_10();
                    function_6();
                    break;
                default:
                    break;
            }
            currentState = 1;
            return;
        }

        public void ChangeStateToEraseIdle()
        {
            switch (currentState)
            {
                case 0:
                    function_11();
                    break;
                case 1:
                    function_7();
                    function_11();
                    break;
                case 3:
                    function_9();
                    break;
                default:
                    break;
            }
            currentState = 2;
            return;
        }

        public void ChangeStateToEraseUsing()
        {
            switch (currentState)
            {
                case 0:
                    function_11();
                    function_8();
                    break;
                case 1:
                    function_7();
                    function_11();
                    function_8();
                    break;
                case 2:
                    function_8();
                    break;
                default:
                    break;
            }
            currentState = 3;
            return;
        }

        public void _TakeOwnership()
        {
            if (VRC.SDKBase.Networking.IsOwner(this.gameObject))
            {
                __0__TakeOwnership__ret = true;
                return;
            }
            else
            {
                VRC.SDKBase.Networking.SetOwner(VRC.SDKBase.Networking.LocalPlayer, this.gameObject);
                __0__TakeOwnership__ret = VRC.SDKBase.Networking.IsOwner(this.gameObject);
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
            if (VRC.SDKBase.Networking.IsOwner(this.gameObject))
            {
                get_objectSync();
                if (!VRC.SDKBase.Utilities.IsValid(__0_get_objectSync__ret))
                {
                    if (VRC.SDKBase.Utilities.IsValid(_alternativeObjectSync))
                    {
                        _alternativeObjectSync.SendCustomEvent(_respawnEventName);
                    }
                }
                else
                {
                    get_objectSync();
                    __0_get_objectSync__ret.Respawn();
                }
            }
            return;
        }

        public void _Clear()
        {
            manager.SetProgramVariable("__5_penId__param", penId);
            manager.SendCustomEvent("__0_Clear");
            return;
        }

        public void _EraseOwnInk()
        {
            _TakeOwnership();
            function_18();
            return;
        }

        public void _UndoDraw()
        {
            _TakeOwnership();
            function_19();
            return;
        }

        void function_6()
        {
            trailRenderer.gameObject.SetActive(true);
            return;
        }

        void function_7()
        {
            System.Int32 __lcl_inkId_SystemInt32_0;
            UnityEngine.TrailRenderer __intnl_UnityEngineTrailRenderer_0;
            UnityEngine.Vector3[] __lcl_data_UnityEngineVector3Array_0;
            UnityEngine.Vector3 __lcl_inkIdVector_UnityEngineVector3_0;
            if (isUser)
            {
                penManager.SendCustomEvent("get_InkId");
                __lcl_inkId_SystemInt32_0 = System.Convert.ToInt32(penManager.GetProgramVariable("__0_get_InkId__ret"));
                __5__intnlparam = __lcl_inkId_SystemInt32_0;
                function_29();
                __lcl_inkIdVector_UnityEngineVector3_0 = __4__intnlparam;
                __intnl_UnityEngineTrailRenderer_0 = trailRenderer;
                get_localPlayerIdVector();
                __0_trailRenderer__param = __intnl_UnityEngineTrailRenderer_0;
                __1_mode__param = 2;
                __0_inkIdVector__param = __lcl_inkIdVector_UnityEngineVector3_0;
                __0_ownerIdVector__param = __0_get_localPlayerIdVector__ret;
                PackData();
                __lcl_data_UnityEngineVector3Array_0 = __0___0_PackData__ret;
                __0_inkId__param = __lcl_inkId_SystemInt32_0;
                function_12();
                penManager.SendCustomEvent("_IncrementInkId");
                __4_data__param = __lcl_data_UnityEngineVector3Array_0;
                __0__SendData();
            }
            trailRenderer.gameObject.SetActive(false);
            trailRenderer.Clear();
            return;
        }

        void PackData()
        {
            System.Single __intnl_SystemSingle_24;
            System.Int32 __lcl_positionCount_SystemInt32_0;
            System.Int32 __lcl_modeAsInt_SystemInt32_0;
            UnityEngine.Vector3[] __lcl_positions_UnityEngineVector3Array_0;
            UnityEngine.Vector3[] __lcl_data_UnityEngineVector3Array_1;
            if (VRC.SDKBase.Utilities.IsValid(__0_trailRenderer__param))
            {
                __lcl_positionCount_SystemInt32_0 = __0_trailRenderer__param.positionCount;
                if (__lcl_positionCount_SystemInt32_0 == 0)
                {
                    __0___0_PackData__ret = null;
                    return;
                }
                else
                {
                    __lcl_positions_UnityEngineVector3Array_0 = new UnityEngine.Vector3[](__lcl_positionCount_SystemInt32_0);
                    System.Array.Reverse(__lcl_positions_UnityEngineVector3Array_0);
                    __11__intnlparam = __1_mode__param;
                    function_32();
                    __lcl_data_UnityEngineVector3Array_1 = new UnityEngine.Vector3[](__lcl_positionCount_SystemInt32_0 + __10__intnlparam);
                    System.Array.Copy(__lcl_positions_UnityEngineVector3Array_0, __lcl_data_UnityEngineVector3Array_1, __lcl_positionCount_SystemInt32_0);
                    __lcl_modeAsInt_SystemInt32_0 = __1_mode__param;
                    get_localPlayerId();
                    __11__intnlparam = __1_mode__param;
                    function_32();
                    __12__intnlparam = __lcl_data_UnityEngineVector3Array_1;
                    __13__intnlparam = 0;
                    __14__intnlparam =
                        new UnityEngine.Vector3(System.Convert.ToSingle(__0_get_localPlayerId__ret), System.Convert.ToSingle(__lcl_modeAsInt_SystemInt32_0),
                                                System.Convert.ToSingle(__10__intnlparam));
                    function_33();
                    get_penIdVector();
                    __12__intnlparam = __lcl_data_UnityEngineVector3Array_1;
                    __13__intnlparam = 1;
                    __14__intnlparam = __0_get_penIdVector__ret;
                    function_33();
                    __12__intnlparam = __lcl_data_UnityEngineVector3Array_1;
                    __13__intnlparam = 2;
                    __14__intnlparam = __0_inkIdVector__param;
                    function_33();
                    __12__intnlparam = __lcl_data_UnityEngineVector3Array_1;
                    __13__intnlparam = 3;
                    __14__intnlparam = __0_ownerIdVector__param;
                    function_33();
                    if (enabledLateSync)
                    {
                        __intnl_SystemSingle_24 = 1.0f;
                    }
                    else
                    {
                        __intnl_SystemSingle_24 = 0.0f;
                    }
                    __12__intnlparam = __lcl_data_UnityEngineVector3Array_1;
                    __13__intnlparam = 4;
                    __14__intnlparam =
                        new UnityEngine.Vector3(System.Convert.ToSingle(inkMeshLayer), System.Convert.ToSingle(inkColliderLayer), __intnl_SystemSingle_24);
                    function_33();
                    __0___0_PackData__ret = __lcl_data_UnityEngineVector3Array_1;
                    return;
                }
            }
            else
            {
                __0___0_PackData__ret = null;
                return;
            }
        }

        public void __0__PackData()
        {
            System.Int32 __intnl_SystemInt32_15;
            System.Int32 __lcl_positionCount_SystemInt32_1;
            System.Int32 __lcl_modeAsInt_SystemInt32_1;
            System.Int32 __intnl_SystemInt32_14;
            UnityEngine.Vector3[] __lcl_positions_UnityEngineVector3Array_1;
            System.Int32 __lcl_inkMeshLayer_SystemInt32_0;
            System.Int32 __lcl_inkColliderLayer_SystemInt32_0;
            UnityEngine.Vector3[] __lcl_data_UnityEngineVector3Array_2;
            if (VRC.SDKBase.Utilities.IsValid(__0_lineRenderer__param))
            {
                __lcl_positionCount_SystemInt32_1 = __0_lineRenderer__param.positionCount;
                if (__lcl_positionCount_SystemInt32_1 == 0)
                {
                    __0___0__PackData__ret = null;
                    return;
                }
                else
                {
                    __lcl_positions_UnityEngineVector3Array_1 = new UnityEngine.Vector3[](__lcl_positionCount_SystemInt32_1);
                    __11__intnlparam = __2_mode__param;
                    function_32();
                    __lcl_data_UnityEngineVector3Array_2 = new UnityEngine.Vector3[](__lcl_positionCount_SystemInt32_1 + __10__intnlparam);
                    System.Array.Copy(__lcl_positions_UnityEngineVector3Array_1, __lcl_data_UnityEngineVector3Array_2, __lcl_positionCount_SystemInt32_1);
                    __lcl_inkMeshLayer_SystemInt32_0 = __0_lineRenderer__param.gameObject.layer;
                    __lcl_inkColliderLayer_SystemInt32_0 =
                        __0_lineRenderer__param.transform
                            .GetComponentInChildren(
                                true, null /* "UnityEngine.MeshCollider, UnityEngine.PhysicsModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */)
                            .gameObject.layer;
                    __lcl_modeAsInt_SystemInt32_1 = __2_mode__param;
                    get_localPlayerId();
                    __11__intnlparam = __2_mode__param;
                    __intnl_SystemInt32_14 = __0_get_localPlayerId__ret;
                    function_32();
                    __12__intnlparam = __lcl_data_UnityEngineVector3Array_2;
                    __13__intnlparam = 0;
                    __14__intnlparam =
                        (UnityEngine.Vector3Int) new UnityEngine.Vector3Int(__intnl_SystemInt32_14, __lcl_modeAsInt_SystemInt32_1, __10__intnlparam);
                    function_33();
                    get_penIdVector();
                    __12__intnlparam = __lcl_data_UnityEngineVector3Array_2;
                    __13__intnlparam = 1;
                    __14__intnlparam = __0_get_penIdVector__ret;
                    function_33();
                    __12__intnlparam = __lcl_data_UnityEngineVector3Array_2;
                    __13__intnlparam = 2;
                    __14__intnlparam = __1_inkIdVector__param;
                    function_33();
                    __12__intnlparam = __lcl_data_UnityEngineVector3Array_2;
                    __13__intnlparam = 3;
                    __14__intnlparam = __1_ownerIdVector__param;
                    function_33();
                    if (enabledLateSync)
                    {
                        __intnl_SystemInt32_15 = 1;
                    }
                    else
                    {
                        __intnl_SystemInt32_15 = 0;
                    }
                    __12__intnlparam = __lcl_data_UnityEngineVector3Array_2;
                    __13__intnlparam = 4;
                    __14__intnlparam = (UnityEngine.Vector3Int) new UnityEngine.Vector3Int(__lcl_inkMeshLayer_SystemInt32_0,
                                                                                           __lcl_inkColliderLayer_SystemInt32_0, __intnl_SystemInt32_15);
                    function_33();
                    __0___0__PackData__ret = __lcl_data_UnityEngineVector3Array_2;
                    return;
                }
            }
            else
            {
                __0___0__PackData__ret = null;
                return;
            }
        }

        public void __0__SendData()
        {
            penManager.SetProgramVariable("__0_data__param", __4_data__param);
            penManager.SendCustomEvent("__0__SendData");
            return;
        }

        void function_8()
        {
            isPointerEnabled = true;
            if (VRC.SDKBase.Utilities.IsValid(pointerRenderer))
            {
                pointerRenderer.sharedMaterial = pointerMaterialActive;
            }
            return;
        }

        void function_9()
        {
            isPointerEnabled = false;
            if (VRC.SDKBase.Utilities.IsValid(pointerRenderer))
            {
                pointerRenderer.sharedMaterial = pointerMaterialNormal;
            }
            return;
        }

        void function_10()
        {
            function_9();
            pointer.gameObject.SetActive(false);
            return;
        }

        void function_11()
        {
            pointer.gameObject.SetActive(true);
            return;
        }

        public void __0__UnpackData()
        {
            System.Int32 __lcl_mode_SystemInt32_0;
            System.Boolean __intnl_SystemBoolean_71;
            __16__intnlparam = __5_data__param;
            function_34();
            __lcl_mode_SystemInt32_0 = __15__intnlparam;
            __intnl_SystemBoolean_71 = __0_targetMode__param != 1;
            if (__intnl_SystemBoolean_71)
            {
                __intnl_SystemBoolean_71 = __lcl_mode_SystemInt32_0 != __0_targetMode__param;
            }
            if (__intnl_SystemBoolean_71)
            {
                return;
            }
            else
            {
                switch (__lcl_mode_SystemInt32_0)
                {
                    case 2:
                        __7_data__param = __5_data__param;
                        function_13();
                        break;
                    case 3:
                        __8_data__param = __5_data__param;
                        function_20();
                        break;
                    case 4:
                        __9_data__param = __5_data__param;
                        function_21();
                        break;
                    default:
                        break;
                }
                return;
            }
        }

        public void __0__EraseAbandonedInk()
        {
            System.Int32 __lcl_mode_SystemInt32_1;
            __16__intnlparam = __6_data__param;
            function_34();
            __lcl_mode_SystemInt32_1 = __15__intnlparam;
            if (__lcl_mode_SystemInt32_1 != 2)
            {
                return;
            }
            else
            {
                __8_data__param = __6_data__param;
                function_20();
                return;
            }
        }

        void function_12()
        {
            if (localInkHistory.Count > 1024)
            {
                localInkHistory.RemoveAt(0);
            }
            localInkHistory.Add((VRC.SDK3.Data.DataToken)__0_inkId__param);
            return;
        }

        void TryGetLastLocalInk()
        {
            VRC.SDK3.Data.DataToken __lcl_inkIdToken_VRCSDK3DataDataToken_0;
            System.Int32 __lcl_i_SystemInt32_1;
            __lcl_i_SystemInt32_1 = localInkHistory.Count - 1;
            while (__lcl_i_SystemInt32_1 >= 0)
            {
                if (localInkHistory.TryGetValue(__lcl_i_SystemInt32_1, null /* 6 */, out __lcl_inkIdToken_VRCSDK3DataDataToken_0))
                {
                    __1_inkId__param = __lcl_inkIdToken_VRCSDK3DataDataToken_0.Int;
                    manager.SetProgramVariable("__1_penId__param", penId);
                    manager.SetProgramVariable("__0_inkId__param", __1_inkId__param);
                    manager.SendCustomEvent("__0_HasInk");
                    if (manager.GetProgramVariable("__0___0_HasInk__ret"))
                    {
                        __0___0_TryGetLastLocalInk__ret = true;
                        return;
                    }
                    else
                    {
                        localInkHistory.RemoveAt(__lcl_i_SystemInt32_1);
                        __lcl_i_SystemInt32_1 = __lcl_i_SystemInt32_1 - 1;
                        continue;
                    }
                }
                else
                {
                    goto label_bb_000053d0;
                }
            }
            __1_inkId__param = 0;
            __0___0_TryGetLastLocalInk__ret = false;
            return;
        label_bb_000053d0:
        }

        void function_13()
        {
            System.Int32 __lcl_positionCount_SystemInt32_2;
            System.Int32 __lcl_inkId_SystemInt32_1;
            UnityEngine.Vector3 __lcl_inkInfo_UnityEngineVector3_0;
            System.Int32 __lcl_penId_SystemInt32_0;
            UnityEngine.Vector3 __lcl_inkIdVector_UnityEngineVector3_1;
            UnityEngine.Vector3 __lcl_playerIdVector_UnityEngineVector3_0;
            UnityEngine.GameObject __lcl_lineInstance_UnityEngineGameObject_0;
            UnityEngine.Transform __intnl_UnityEngineTransform_22;
            UnityEngine.Vector3 __lcl_penIdVector_UnityEngineVector3_0;
            UnityEngine.LineRenderer __lcl_line_UnityEngineLineRenderer_0;
            __18__intnlparam = __7_data__param;
            __19__intnlparam = 1;
            function_35();
            __lcl_penIdVector_UnityEngineVector3_0 = __17__intnlparam;
            __18__intnlparam = __7_data__param;
            __19__intnlparam = 2;
            function_35();
            __lcl_inkIdVector_UnityEngineVector3_1 = __17__intnlparam;
            __21__intnlparam = __lcl_penIdVector_UnityEngineVector3_0;
            function_36();
            __lcl_penId_SystemInt32_0 = __20__intnlparam;
            __21__intnlparam = __lcl_inkIdVector_UnityEngineVector3_1;
            function_36();
            __lcl_inkId_SystemInt32_1 = __20__intnlparam;
            manager.SetProgramVariable("__1_penId__param", __lcl_penId_SystemInt32_0);
            manager.SetProgramVariable("__0_inkId__param", __lcl_inkId_SystemInt32_1);
            manager.SendCustomEvent("__0_HasInk");
            if (manager.GetProgramVariable("__0___0_HasInk__ret"))
            {
                return;
            }
            else
            {
                __18__intnlparam = __7_data__param;
                __19__intnlparam = 3;
                function_35();
                __lcl_playerIdVector_UnityEngineVector3_0 = __17__intnlparam;
                __23__intnlparam = inkPrefab.gameObject;
                function_37();
                __lcl_lineInstance_UnityEngineGameObject_0 = __22__intnlparam;
                __lcl_lineInstance_UnityEngineGameObject_0.name = System.String.Format("{0} ({1})", "Ink", __lcl_inkId_SystemInt32_1);
                __25__intnlparam = __lcl_lineInstance_UnityEngineGameObject_0;
                __26__intnlparam = __lcl_penIdVector_UnityEngineVector3_0;
                __27__intnlparam = __lcl_inkIdVector_UnityEngineVector3_1;
                __28__intnlparam = __lcl_playerIdVector_UnityEngineVector3_0;
                function_38();
                if (__24__intnlparam)
                {
                    manager.SetProgramVariable("__2_penId__param", __lcl_penId_SystemInt32_0);
                    manager.SetProgramVariable("__1_inkId__param", __lcl_inkId_SystemInt32_1);
                    manager.SetProgramVariable("__0_inkInstance__param", __lcl_lineInstance_UnityEngineGameObject_0);
                    manager.SendCustomEvent("__0_SetInk");
                    __18__intnlparam = __7_data__param;
                    __19__intnlparam = 4;
                    function_35();
                    __lcl_inkInfo_UnityEngineVector3_0 = __17__intnlparam;
                    __lcl_lineInstance_UnityEngineGameObject_0.layer =
                        System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__lcl_inkInfo_UnityEngineVector3_0.x)));
                    __lcl_lineInstance_UnityEngineGameObject_0.transform
                        .GetComponentInChildren(
                            true, null /* "UnityEngine.MeshCollider, UnityEngine.PhysicsModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */)
                        .gameObject.layer = System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__lcl_inkInfo_UnityEngineVector3_0.y)));
                    if (System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__lcl_inkInfo_UnityEngineVector3_0.z))) == 1)
                    {
                        __intnl_UnityEngineTransform_22 = inkPoolSynced;
                    }
                    else
                    {
                        __intnl_UnityEngineTransform_22 = inkPoolNotSynced;
                    }
                    __2__intnlparam = __lcl_lineInstance_UnityEngineGameObject_0.transform;
                    __3__intnlparam = __intnl_UnityEngineTransform_22;
                    function_28();
                    __30__intnlparam = __7_data__param;
                    function_39();
                    __lcl_positionCount_SystemInt32_2 = __7_data__param.Length - __29__intnlparam;
                    __lcl_line_UnityEngineLineRenderer_0 = __lcl_lineInstance_UnityEngineGameObject_0.transform.GetComponent(
                        null /* "UnityEngine.LineRenderer, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
                    __lcl_line_UnityEngineLineRenderer_0.positionCount = __lcl_positionCount_SystemInt32_2;
                    __lcl_line_UnityEngineLineRenderer_0.SetPositions(__7_data__param);
                    if (isRoundedTrailShader)
                    {
                        if (VRC.SDKBase.Utilities.IsValid(propertyBlock))
                        {
                            propertyBlock.Clear();
                        }
                        else
                        {
                            propertyBlock = new UnityEngine.MaterialPropertyBlock();
                        }
                        __lcl_line_UnityEngineLineRenderer_0.GetPropertyBlock(propertyBlock);
                        propertyBlock.SetFloat("_Width", inkWidth);
                        __lcl_line_UnityEngineLineRenderer_0.SetPropertyBlock(propertyBlock);
                    }
                    else
                    {
                        __lcl_line_UnityEngineLineRenderer_0.widthMultiplier = inkWidth;
                    }
                    __1_lineRenderer__param = __lcl_line_UnityEngineLineRenderer_0;
                    function_14();
                    __lcl_lineInstance_UnityEngineGameObject_0.SetActive(true);
                    return;
                }
                else
                {
                    __1_o__param = System.String.Format("Failed TrySetIdFromInk pen: {0}, ink: {1}", __lcl_penId_SystemInt32_0, __lcl_inkId_SystemInt32_1);
                    function_24();
                    UnityEngine.Object.Destroy(__lcl_lineInstance_UnityEngineGameObject_0);
                    return;
                }
            }
        }

        void function_14()
        {
            UnityEngine.Mesh __lcl_mesh_UnityEngineMesh_0;
            UnityEngine.MeshCollider __lcl_inkCollider_UnityEngineMeshCollider_0;
            System.Single __lcl_tmpWidthMultiplier_SystemSingle_0;
            __lcl_inkCollider_UnityEngineMeshCollider_0 = __1_lineRenderer__param.transform.GetComponentInChildren(
                true, null /* "UnityEngine.MeshCollider, UnityEngine.PhysicsModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
            __lcl_inkCollider_UnityEngineMeshCollider_0.name = "InkCollider";
            __lcl_mesh_UnityEngineMesh_0 = new UnityEngine.Mesh();
            __lcl_tmpWidthMultiplier_SystemSingle_0 = __1_lineRenderer__param.widthMultiplier;
            __1_lineRenderer__param.widthMultiplier = inkWidth;
            __1_lineRenderer__param.BakeMesh(__lcl_mesh_UnityEngineMesh_0, false);
            __1_lineRenderer__param.widthMultiplier = __lcl_tmpWidthMultiplier_SystemSingle_0;
            __lcl_inkCollider_UnityEngineMeshCollider_0.transform
                .GetComponent(null /* "UnityEngine.MeshCollider, UnityEngine.PhysicsModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */)
                .sharedMesh = __lcl_mesh_UnityEngineMesh_0;
            __lcl_inkCollider_UnityEngineMeshCollider_0.gameObject.SetActive(true);
            return;
        }

        void function_15()
        {
            UnityEngine.Vector3[] __lcl_data_UnityEngineVector3Array_3;
            __11__intnlparam = 3;
            function_32();
            __lcl_data_UnityEngineVector3Array_3 = new UnityEngine.Vector3[](__10__intnlparam);
            get_localPlayerId();
            __11__intnlparam = 3;
            function_32();
            __12__intnlparam = __lcl_data_UnityEngineVector3Array_3;
            __13__intnlparam = 0;
            __14__intnlparam = new UnityEngine.Vector3(System.Convert.ToSingle(__0_get_localPlayerId__ret), 3.0f, System.Convert.ToSingle(__10__intnlparam));
            function_33();
            __12__intnlparam = __lcl_data_UnityEngineVector3Array_3;
            __13__intnlparam = 1;
            __14__intnlparam = __0_penIdVector__param;
            function_33();
            __12__intnlparam = __lcl_data_UnityEngineVector3Array_3;
            __13__intnlparam = 2;
            __14__intnlparam = __2_inkIdVector__param;
            function_33();
            __4_data__param = __lcl_data_UnityEngineVector3Array_3;
            __0__SendData();
            return;
        }

        void function_16()
        {
            UnityEngine.Vector3 __intnl_UnityEngineVector3_26;
            __5__intnlparam = __0_penId__param;
            function_29();
            __5__intnlparam = __2_inkId__param;
            __intnl_UnityEngineVector3_26 = __4__intnlparam;
            function_29();
            __0_penIdVector__param = __intnl_UnityEngineVector3_26;
            __2_inkIdVector__param = __4__intnlparam;
            function_15();
            return;
        }

        void function_17()
        {
            System.Boolean __intnl_SystemBoolean_83;
            UnityEngine.Vector3 __lcl__discard_UnityEngineVector3_0;
            UnityEngine.Vector3 __lcl_penIdVector_UnityEngineVector3_1;
            UnityEngine.Vector3 __lcl_inkIdVector_UnityEngineVector3_2;
            __intnl_SystemBoolean_83 = VRC.SDKBase.Utilities.IsValid(__0_ink__param);
            if (__intnl_SystemBoolean_83)
            {
                __32__intnlparam = __0_ink__param;
                __33__intnlparam = __lcl_penIdVector_UnityEngineVector3_1;
                __34__intnlparam = __lcl_inkIdVector_UnityEngineVector3_2;
                __35__intnlparam = __lcl__discard_UnityEngineVector3_0;
                function_40();
                __lcl_penIdVector_UnityEngineVector3_1 = __33__intnlparam;
                __lcl_inkIdVector_UnityEngineVector3_2 = __34__intnlparam;
                __lcl__discard_UnityEngineVector3_0 = __35__intnlparam;
                __intnl_SystemBoolean_83 = __31__intnlparam;
            }
            if (__intnl_SystemBoolean_83)
            {
                __0_penIdVector__param = __lcl_penIdVector_UnityEngineVector3_1;
                __2_inkIdVector__param = __lcl_inkIdVector_UnityEngineVector3_2;
                function_15();
            }
            return;
        }

        void function_18()
        {
            UnityEngine.Vector3[] __lcl_data_UnityEngineVector3Array_4;
            __11__intnlparam = 4;
            function_32();
            __lcl_data_UnityEngineVector3Array_4 = new UnityEngine.Vector3[](__10__intnlparam);
            get_localPlayerId();
            __11__intnlparam = 4;
            function_32();
            __12__intnlparam = __lcl_data_UnityEngineVector3Array_4;
            __13__intnlparam = 0;
            __14__intnlparam = new UnityEngine.Vector3(System.Convert.ToSingle(__0_get_localPlayerId__ret), 4.0f, System.Convert.ToSingle(__10__intnlparam));
            function_33();
            get_penIdVector();
            __12__intnlparam = __lcl_data_UnityEngineVector3Array_4;
            __13__intnlparam = 1;
            __14__intnlparam = __0_get_penIdVector__ret;
            function_33();
            get_localPlayerIdVector();
            __12__intnlparam = __lcl_data_UnityEngineVector3Array_4;
            __13__intnlparam = 3;
            __14__intnlparam = __0_get_localPlayerIdVector__ret;
            function_33();
            __4_data__param = __lcl_data_UnityEngineVector3Array_4;
            __0__SendData();
            return;
        }

        void function_19()
        {
            System.Int32 __lcl_inkId_SystemInt32_2;
            __1_inkId__param = __lcl_inkId_SystemInt32_2;
            TryGetLastLocalInk();
            __lcl_inkId_SystemInt32_2 = __1_inkId__param;
            if (__0___0_TryGetLastLocalInk__ret)
            {
                __0_penId__param = penId;
                __2_inkId__param = __lcl_inkId_SystemInt32_2;
                function_16();
                return;
            }
            else
            {
                return;
            }
        }

        void function_20()
        {
            System.Int32 __lcl_inkId_SystemInt32_3;
            System.Int32 __lcl_penId_SystemInt32_1;
            UnityEngine.Vector3 __lcl_inkIdVector_UnityEngineVector3_3;
            UnityEngine.Vector3 __lcl_penIdVector_UnityEngineVector3_2;
            __11__intnlparam = 3;
            function_32();
            if (__8_data__param.Length < __10__intnlparam)
            {
                return;
            }
            else
            {
                __18__intnlparam = __8_data__param;
                __19__intnlparam = 1;
                function_35();
                __lcl_penIdVector_UnityEngineVector3_2 = __17__intnlparam;
                __18__intnlparam = __8_data__param;
                __19__intnlparam = 2;
                function_35();
                __lcl_inkIdVector_UnityEngineVector3_3 = __17__intnlparam;
                __21__intnlparam = __lcl_penIdVector_UnityEngineVector3_2;
                function_36();
                __lcl_penId_SystemInt32_1 = __20__intnlparam;
                __21__intnlparam = __lcl_inkIdVector_UnityEngineVector3_3;
                function_36();
                __lcl_inkId_SystemInt32_3 = __20__intnlparam;
                manager.SetProgramVariable("__3_penId__param", __lcl_penId_SystemInt32_1);
                manager.SetProgramVariable("__2_inkId__param", __lcl_inkId_SystemInt32_3);
                manager.SendCustomEvent("__0_RemoveInk");
                return;
            }
        }

        void function_21()
        {
            UnityEngine.Vector3 __lcl_ownerIdVector_UnityEngineVector3_0;
            System.Int32 __lcl_penId_SystemInt32_2;
            __11__intnlparam = 4;
            function_32();
            if (__9_data__param.Length < __10__intnlparam)
            {
                return;
            }
            else
            {
                __18__intnlparam = __9_data__param;
                __19__intnlparam = 3;
                function_35();
                __lcl_ownerIdVector_UnityEngineVector3_0 = __17__intnlparam;
                get_penIdVector();
                __21__intnlparam = __0_get_penIdVector__ret;
                function_36();
                __lcl_penId_SystemInt32_2 = __20__intnlparam;
                manager.SetProgramVariable("__4_penId__param", __lcl_penId_SystemInt32_2);
                manager.SetProgramVariable("__0_ownerIdVector__param", __lcl_ownerIdVector_UnityEngineVector3_0);
                manager.SendCustomEvent("__0_RemoveUserInk");
                return;
            }
        }

        void function_22()
        {
            UnityEngine.Component __lcl_udonComponent_UnityEngineComponent_0;
            System.Int32 __intnl_SystemInt32_27;
            System.Int32 __lcl_count_SystemInt32_1;
            UnityEngine.Component[] __lcl_udonComponents_UnityEngineComponentArray_0;
            VRC.Udon.UdonBehaviour __lcl_udon_VRCUdonUdonBehaviour_0;
            System.Int32 __lcl_i_SystemInt32_2;
            System.Int32 __intnl_SystemInt32_26;
            UnityEngine.Collider __lcl_other_UnityEngineCollider_1;
            get_pointerRadius();
            __lcl_count_SystemInt32_1 = UnityEngine.Physics.OverlapSphereNonAlloc(pointer.position, __0_get_pointerRadius__ret, results32, -1, null /* 2 */);
            __lcl_i_SystemInt32_2 = 0;
            while (__lcl_i_SystemInt32_2 < __lcl_count_SystemInt32_1)
            {
                __lcl_other_UnityEngineCollider_1 = results32.Get(__lcl_i_SystemInt32_2);
                if (VRC.SDKBase.Utilities.IsValid(__lcl_other_UnityEngineCollider_1))
                {
                    __lcl_udonComponents_UnityEngineComponentArray_0 = __lcl_other_UnityEngineCollider_1.GetComponents(
                        null /* "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
                    __intnl_SystemInt32_26 = __lcl_udonComponents_UnityEngineComponentArray_0.Length;
                    __intnl_SystemInt32_27 = 0;
                    if (__intnl_SystemInt32_27 < __intnl_SystemInt32_26)
                    {
                        __lcl_udonComponent_UnityEngineComponent_0 = __lcl_udonComponents_UnityEngineComponentArray_0.Get(__intnl_SystemInt32_27);
                        if (VRC.SDKBase.Utilities.IsValid(__lcl_udonComponent_UnityEngineComponent_0))
                        {
                            __lcl_udon_VRCUdonUdonBehaviour_0 = __lcl_udonComponent_UnityEngineComponent_0;
                            if (!__lcl_udon_VRCUdonUdonBehaviour_0.DisableInteractive)
                            {
                                __lcl_udon_VRCUdonUdonBehaviour_0.SendCustomEvent("_interact");
                            }
                        }
                        __intnl_SystemInt32_27 = __intnl_SystemInt32_27 + 1;
                        goto label_bb_00006a64;
                    }
                    else
                    {
                        goto label_bb_00006b70;
                    }
                }
            label_bb_00006b70:
                results32.Set(__lcl_i_SystemInt32_2, null);
                __lcl_i_SystemInt32_2 = __lcl_i_SystemInt32_2 + 1;
            }
            __intnl_SystemInt32_26 = results32.Length;
            System.Array.Clear(results32, 0, __intnl_SystemInt32_26);
            return;
        label_bb_00006a64:
        }

        void function_23()
        {
            get_logPrefix();
            UnityEngine.Debug.Log(System.String.Format("{0}{1}", __0_get_logPrefix__ret, __0_o__param), this);
            return;
        }

        void function_24()
        {
            get_logPrefix();
            UnityEngine.Debug.LogWarning(System.String.Format("{0}{1}", __0_get_logPrefix__ret, __1_o__param), this);
            return;
        }

        void function_25()
        {
            get_logPrefix();
            UnityEngine.Debug.LogError(System.String.Format("{0}{1}", __0_get_logPrefix__ret, __2_o__param), this);
            return;
        }

        void function_26()
        {
            __37__intnlparam = __0_c__param;
            function_41();
            __0___0_ColorBeginTag__ret = System.String.Format("<color=\"#{0}\">", __36__intnlparam);
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
                function_26();
                __gintnl_SystemObjectArray_0.Set(0, __0___0_ColorBeginTag__ret);
                __gintnl_SystemObjectArray_0.Set(1, "QvPen");
                __gintnl_SystemObjectArray_0.Set(2, "Udon");
                __gintnl_SystemObjectArray_0.Set(3, "QvPen_Pen");
                __gintnl_SystemObjectArray_0.Set(4, "</color>");
                _logPrefix = System.String.Format("[{0}{1}.{2}.{3}{4}] ", __gintnl_SystemObjectArray_0);
                __0_get_logPrefix__ret = _logPrefix;
            }
            return;
        }

        void function_27()
        {
            System.Int32 __lcl_y_SystemInt32_0;
            System.Int32 __lcl_z_SystemInt32_0;
            System.Int32 __lcl_x_SystemInt32_0;
            __lcl_x_SystemInt32_0 = __1__intnlparam;
            __lcl_y_SystemInt32_0 = __lcl_x_SystemInt32_0 / 360;
            __lcl_z_SystemInt32_0 = __lcl_y_SystemInt32_0 / 360;
            __0__intnlparam =
                new UnityEngine.Vector3(System.Convert.ToSingle(__lcl_x_SystemInt32_0 % 360), System.Convert.ToSingle(__lcl_y_SystemInt32_0 % 360),
                                        System.Convert.ToSingle(__lcl_z_SystemInt32_0 % 360)) /
                4.0f;
            return;
        }

        void function_28()
        {
            if (VRC.SDKBase.Utilities.IsValid(__2__intnlparam))
            {
                __2__intnlparam.SetParent(__3__intnlparam);
                __2__intnlparam.SetLocalPositionAndRotation(__const_UnityEngineVector3_1, __const_UnityEngineQuaternion_0);
                __2__intnlparam.localScale = __const_UnityEngineVector3_0;
                return;
            }
            else
            {
                return;
            }
        }

        void function_29()
        {
            __4__intnlparam = new UnityEngine.Vector3(System.Convert.ToSingle(__5__intnlparam >> 24 & 255),
                                                      System.Convert.ToSingle(__5__intnlparam >> 12 & 4095), System.Convert.ToSingle(__5__intnlparam & 4095));
            return;
        }

        void function_30()
        {
            System.Int32 __intnl_SystemInt32_37;
            VRC.Udon.UdonBehaviour __lcl_behaviour_VRCUdonUdonBehaviour_0;
            System.Int64 __lcl_targetID_SystemInt64_0;
            System.Object __lcl_idValue_SystemObject_0;
            UnityEngine.Component[] __lcl_udonBehaviours_UnityEngineComponentArray_0;
            System.Boolean __intnl_SystemBoolean_98;
            __lcl_udonBehaviours_UnityEngineComponentArray_0 =
                __7__intnlparam.GetComponents(null /* "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
            __lcl_targetID_SystemInt64_0 = 8320864560273335438;
            __intnl_SystemInt32_37 = 0;
            while (__intnl_SystemInt32_37 < __lcl_udonBehaviours_UnityEngineComponentArray_0.Length)
            {
                __lcl_behaviour_VRCUdonUdonBehaviour_0 = __lcl_udonBehaviours_UnityEngineComponentArray_0.Get(__intnl_SystemInt32_37);
                if (__lcl_behaviour_VRCUdonUdonBehaviour_0.GetProgramVariableType("__refl_typeid") == null)
                {
                    goto label_bb_000074a4;
                }
                else
                {
                    __lcl_idValue_SystemObject_0 = __lcl_behaviour_VRCUdonUdonBehaviour_0.GetProgramVariable("__refl_typeid");
                    __intnl_SystemBoolean_98 = __lcl_idValue_SystemObject_0 != null;
                    if (__intnl_SystemBoolean_98)
                    {
                        __intnl_SystemBoolean_98 = System.Convert.ToInt64(__lcl_idValue_SystemObject_0) == __lcl_targetID_SystemInt64_0;
                    }
                    if (__intnl_SystemBoolean_98)
                    {
                        __6__intnlparam = __lcl_behaviour_VRCUdonUdonBehaviour_0;
                        return;
                    }
                    else
                    {
                        goto label_bb_000074a4;
                    }
                }
            }
            __6__intnlparam = null;
            return;
        label_bb_000074a4:
            __intnl_SystemInt32_37 = __intnl_SystemInt32_37 + 1;
            goto label_bb_00007314;
        label_bb_00007314:
        }

        void function_31()
        {
            switch (__9__intnlparam)
            {
                case 0:
                    __8__intnlparam = "PenIdle";
                    return;
                case 1:
                    __8__intnlparam = "PenUsing";
                    return;
                case 2:
                    __8__intnlparam = "EraserIdle";
                    return;
                case 3:
                    __8__intnlparam = "EraserUsing";
                    return;
                default:
                    __8__intnlparam = "(QvPen_Pen_State.???)";
                    return;
            }
        }

        void function_32()
        {
            switch (__11__intnlparam)
            {
                case 0:
                    __10__intnlparam = 0;
                    return;
                case 1:
                    __10__intnlparam = 4;
                    return;
                case 2:
                    __10__intnlparam = 5;
                    return;
                case 3:
                    __10__intnlparam = 4;
                    return;
                case 4:
                    __10__intnlparam = 4;
                    return;
                default:
                    __10__intnlparam = 0;
                    return;
            }
        }

        void function_33()
        {
            System.Boolean __intnl_SystemBoolean_103;
            __intnl_SystemBoolean_103 = __12__intnlparam != null;
            if (__intnl_SystemBoolean_103)
            {
                __intnl_SystemBoolean_103 = __12__intnlparam.Length > __13__intnlparam;
            }
            if (__intnl_SystemBoolean_103)
            {
                __12__intnlparam.Set(__12__intnlparam.Length - 1 - __13__intnlparam, __14__intnlparam);
            }
            return;
        }

        void function_34()
        {
            System.Boolean __intnl_SystemBoolean_104;
            __intnl_SystemBoolean_104 = __16__intnlparam != null;
            if (__intnl_SystemBoolean_104)
            {
                __intnl_SystemBoolean_104 = __16__intnlparam.Length > 0;
            }
            if (__intnl_SystemBoolean_104)
            {
                __18__intnlparam = __16__intnlparam;
                __19__intnlparam = 0;
                function_35();
                __15__intnlparam = System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__17__intnlparam.y)));
            }
            else
            {
                __15__intnlparam = 0;
            }
            return;
        }

        void function_35()
        {
            System.Boolean __intnl_SystemBoolean_105;
            __intnl_SystemBoolean_105 = __18__intnlparam != null;
            if (__intnl_SystemBoolean_105)
            {
                __intnl_SystemBoolean_105 = __18__intnlparam.Length > __19__intnlparam;
            }
            if (__intnl_SystemBoolean_105)
            {
                __17__intnlparam = __18__intnlparam.Get(__18__intnlparam.Length - 1 - __19__intnlparam);
            }
            else
            {
                __17__intnlparam = __const_UnityEngineVector3_1;
            }
            return;
        }

        void function_36()
        {
            __20__intnlparam = (System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__21__intnlparam.x))) & 255) << 24 |
                               (System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__21__intnlparam.y))) & 4095) << 12 |
                               System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__21__intnlparam.z))) & 4095;
            return;
        }

        void function_37()
        {
            __22__intnlparam = UnityEngine.Object.Instantiate(__23__intnlparam);
            return;
        }

        void function_38()
        {
            UnityEngine.Transform __lcl_idHolder_UnityEngineTransform_0;
            if (VRC.SDKBase.Utilities.IsValid(__25__intnlparam))
            {
                if (__25__intnlparam.transform.childCount < 2)
                {
                    __24__intnlparam = false;
                    return;
                }
                else
                {
                    __lcl_idHolder_UnityEngineTransform_0 = __25__intnlparam.transform.GetChild(1);
                    if (VRC.SDKBase.Utilities.IsValid(__lcl_idHolder_UnityEngineTransform_0))
                    {
                        __lcl_idHolder_UnityEngineTransform_0.localPosition = __26__intnlparam;
                        __lcl_idHolder_UnityEngineTransform_0.localScale = __27__intnlparam;
                        __lcl_idHolder_UnityEngineTransform_0.localEulerAngles = __28__intnlparam;
                        __24__intnlparam = true;
                        return;
                    }
                    else
                    {
                        __24__intnlparam = false;
                        return;
                    }
                }
            }
            else
            {
                __24__intnlparam = false;
                return;
            }
        }

        void function_39()
        {
            System.Boolean __intnl_SystemBoolean_109;
            __intnl_SystemBoolean_109 = __30__intnlparam != null;
            if (__intnl_SystemBoolean_109)
            {
                __intnl_SystemBoolean_109 = __30__intnlparam.Length > 0;
            }
            if (__intnl_SystemBoolean_109)
            {
                __18__intnlparam = __30__intnlparam;
                __19__intnlparam = 0;
                function_35();
                __29__intnlparam = UnityEngine.Mathf.Clamp(System.Convert.ToInt32(System.Math.Truncate(System.Convert.ToDouble(__17__intnlparam.z))), 0,
                                                           __30__intnlparam.Length);
            }
            else
            {
                __29__intnlparam = 0;
            }
            return;
        }

        void function_40()
        {
            UnityEngine.Transform __lcl_idHolder_UnityEngineTransform_1;
            if (VRC.SDKBase.Utilities.IsValid(__32__intnlparam))
            {
                if (__32__intnlparam.transform.childCount < 2)
                {
                    __33__intnlparam = __const_UnityEngineVector3_1;
                    __34__intnlparam = __const_UnityEngineVector3_1;
                    __35__intnlparam = __const_UnityEngineVector3_1;
                    __31__intnlparam = false;
                    return;
                }
                else
                {
                    __lcl_idHolder_UnityEngineTransform_1 = __32__intnlparam.transform.GetChild(1);
                    if (VRC.SDKBase.Utilities.IsValid(__lcl_idHolder_UnityEngineTransform_1))
                    {
                        __33__intnlparam = __lcl_idHolder_UnityEngineTransform_1.localPosition;
                        __34__intnlparam = __lcl_idHolder_UnityEngineTransform_1.localScale;
                        __35__intnlparam = __lcl_idHolder_UnityEngineTransform_1.localEulerAngles;
                        __31__intnlparam = true;
                        return;
                    }
                    else
                    {
                        __33__intnlparam = __const_UnityEngineVector3_1;
                        __34__intnlparam = __const_UnityEngineVector3_1;
                        __35__intnlparam = __const_UnityEngineVector3_1;
                        __31__intnlparam = false;
                        return;
                    }
                }
            }
            else
            {
                __33__intnlparam = __const_UnityEngineVector3_1;
                __34__intnlparam = __const_UnityEngineVector3_1;
                __35__intnlparam = __const_UnityEngineVector3_1;
                __31__intnlparam = false;
                return;
            }
        }

        void function_41()
        {
            __37__intnlparam = __37__intnlparam * 255.0f;
            __36__intnlparam = System.String.Format("{0:x2}{1:x2}{2:x2}", UnityEngine.Mathf.RoundToInt(__37__intnlparam.r),
                                                    UnityEngine.Mathf.RoundToInt(__37__intnlparam.g), UnityEngine.Mathf.RoundToInt(__37__intnlparam.b));
            return;
        }
    }
}