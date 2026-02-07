// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.UdonScript
{
    public class QvPen_PenManager : UdonSharpBehaviour
    {
        System.Boolean allowCallPen = true;
        UnityEngine.Material questInkMaterial = null;
        System.Int32 __1_layer__param = 0;
        System.Boolean _isNetworkSettled = false;
        System.Int32 inkId = 0;
        System.Boolean __0_value__param = false;
        UnityEngine.Material pcInkMaterial = null;
        UnityEngine.GameObject respawnButton = null;
        VRC.Udon.Common.SerializationResult onPostSerializationResult = null /* {"success": false, "byteCount": 0} */;
        System.Int32 inkColliderLayer = 9;
        VRC.Udon.UdonBehaviour pen = null;
        UnityEngine.UI.Text textInUse = null;
        UnityEngine.Vector3[] __0_data__param = null;
        System.Int32 __0_get_InkId__ret = 0;
        UnityEngine.Animations.RotationConstraint clearButtonRotationConstraint = null;
        System.Int32 inkMeshLayer = 0;
        System.Int32 __0_layer__param = 0;
        UnityEngine.Vector3[] __3_value__param = null;
        TMPro.TextMeshProUGUI textInUseTMPU = null;
        UnityEngine.GameObject clearButton = null;
        VRC.SDKBase.VRCPlayerApi onPlayerJoinedPlayer = null;
        TMPro.TextMeshPro textInUseTMP = null;
        System.Boolean isInUseSyncBuffer = false;
        UnityEngine.Shader __0_get_roundedTrailShader__ret = null;
        VRC.Udon.UdonBehaviour __0_listener__param = null;
        UnityEngine.Animations.PositionConstraint clearButtonPositionConstraint = null;
        VRC.SDKBase.VRCPlayerApi onPlayerLeftPlayer = null;
        UnityEngine.Gradient colorGradient = null /* "UnityEngine.Gradient" */;
        UnityEngine.Vector3[] __0_get_syncedData__ret = null;
        UnityEngine.GameObject inUseUI = null;
        System.Boolean __0__TakeOwnership__ret = false;
        System.Boolean __2_value__param = false;
        System.Boolean __0_get_AllowCallPen__ret = false;
        System.Boolean __0_isActive__param = false;
        UnityEngine.Vector3[] _syncedData = null;
        System.Single __0_width__param = 0.0f;
        System.Boolean __0_get_isNetworkSettled__ret = false;
        System.Boolean __1_value__param = false;
        UnityEngine.Shader _roundedTrailShader = null;
        VRC.SDK3.Data.DataList listenerList = null /* [] */;
        UnityEngine.LayerMask surftraceMask = null /* {"value": -1} */;
        System.Single inkWidth = 0.005f;

        public void get_roundedTrailShader()
        {
            __0_get_roundedTrailShader__ret = _roundedTrailShader;
            return;
        }

        public void get_AllowCallPen()
        {
            __0_get_AllowCallPen__ret = allowCallPen;
            return;
        }

        public void _start()
        {
            pen.SetProgramVariable("__0_penManager__param", this);
            pen.SendCustomEvent("__0__Init");
            return;
        }

        public void _onPlayerJoined()
        {
            System.Boolean __intnl_SystemBoolean_0;
            __intnl_SystemBoolean_0 = VRC.SDKBase.Networking.IsOwner(pen.gameObject);
            if (__intnl_SystemBoolean_0)
            {
                pen.SendCustomEvent("get_IsUser");
                __intnl_SystemBoolean_0 = pen.GetProgramVariable("__0_get_IsUser__ret");
            }
            if (__intnl_SystemBoolean_0)
            {
                this.SendCustomNetworkEvent(null /* 0 */, "StartUsing");
            }
            if (onPlayerJoinedPlayer.isLocal)
            {
                if (VRC.SDKBase.Utilities.IsValid(clearButton))
                {
                    clearButtonPositionConstraint = clearButton.transform.GetComponent(null /* "UnityEngine.Animations.PositionConstraint, UnityEngine.AnimationModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
                    clearButtonRotationConstraint = clearButton.transform.GetComponent(null /* "UnityEngine.Animations.RotationConstraint, UnityEngine.AnimationModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null" */);
                    function_1();
                }
            }
            return;
        }

        public void _onPlayerLeft()
        {
            System.Boolean __intnl_SystemBoolean_4;
            __intnl_SystemBoolean_4 = VRC.SDKBase.Networking.IsOwner(pen.gameObject);
            if (__intnl_SystemBoolean_4)
            {
                pen.SendCustomEvent("get_IsUser");
                __intnl_SystemBoolean_4 = !pen.GetProgramVariable("__0_get_IsUser__ret");
            }
            if (__intnl_SystemBoolean_4)
            {
                pen.SendCustomEvent("_onDrop");
            }
            return;
        }

        public void StartUsing()
        {
            System.String __lcl_text_SystemString_0;
            VRC.SDKBase.VRCPlayerApi __lcl_owner_VRCSDKBaseVRCPlayerApi_0;
            pen.SetProgramVariable("isPickedUp", true);
            if (VRC.SDKBase.Utilities.IsValid(respawnButton))
            {
                respawnButton.SetActive(false);
            }
            if (VRC.SDKBase.Utilities.IsValid(clearButton))
            {
                __0_isActive__param = false;
                function_0();
            }
            if (VRC.SDKBase.Utilities.IsValid(inUseUI))
            {
                inUseUI.SetActive(true);
            }
            __lcl_owner_VRCSDKBaseVRCPlayerApi_0 = VRC.SDKBase.Networking.GetOwner(pen.gameObject);
            if (__lcl_owner_VRCSDKBaseVRCPlayerApi_0 != null)
            {
                __lcl_text_SystemString_0 = __lcl_owner_VRCSDKBaseVRCPlayerApi_0.displayName;
            }
            else
            {
                __lcl_text_SystemString_0 = "Occupied";
            }
            if (VRC.SDKBase.Utilities.IsValid(textInUse))
            {
                textInUse.text = __lcl_text_SystemString_0;
            }
            if (VRC.SDKBase.Utilities.IsValid(textInUseTMP))
            {
                textInUseTMP.text = __lcl_text_SystemString_0;
            }
            if (VRC.SDKBase.Utilities.IsValid(textInUseTMPU))
            {
                textInUseTMPU.text = __lcl_text_SystemString_0;
            }
            return;
        }

        public void EndUsing()
        {
            pen.SetProgramVariable("isPickedUp", false);
            if (VRC.SDKBase.Utilities.IsValid(respawnButton))
            {
                respawnButton.SetActive(true);
            }
            if (VRC.SDKBase.Utilities.IsValid(clearButton))
            {
                __0_isActive__param = true;
                function_0();
            }
            if (VRC.SDKBase.Utilities.IsValid(inUseUI))
            {
                inUseUI.SetActive(false);
            }
            if (VRC.SDKBase.Utilities.IsValid(textInUse))
            {
                textInUse.text = System.String.Empty;
            }
            if (VRC.SDKBase.Utilities.IsValid(textInUseTMP))
            {
                textInUseTMP.text = System.String.Empty;
            }
            if (VRC.SDKBase.Utilities.IsValid(textInUseTMPU))
            {
                textInUseTMPU.text = System.String.Empty;
            }
            return;
        }

        void function_0()
        {
            if (VRC.SDKBase.Utilities.IsValid(clearButton))
            {
                clearButton.SetActive(__0_isActive__param);
                if (__0_isActive__param)
                {
                    function_1();
                    return;
                }
                else
                {
                    return;
                }
            }
            else
            {
                return;
            }
        }

        void function_1()
        {
            if (VRC.SDKBase.Utilities.IsValid(clearButtonPositionConstraint))
            {
                clearButtonPositionConstraint.enabled = true;
            }
            if (VRC.SDKBase.Utilities.IsValid(clearButtonRotationConstraint))
            {
                clearButtonRotationConstraint.enabled = true;
            }
            this.SendCustomEventDelayedSeconds("_DisableClearButtonConstraints", 2.0f, null /* 0 */);
            return;
        }

        public void _DisableClearButtonConstraints()
        {
            if (VRC.SDKBase.Utilities.IsValid(clearButtonPositionConstraint))
            {
                clearButtonPositionConstraint.enabled = false;
            }
            if (VRC.SDKBase.Utilities.IsValid(clearButtonRotationConstraint))
            {
                clearButtonRotationConstraint.enabled = false;
            }
            return;
        }

        public void __0__SetWidth()
        {
            inkWidth = __0_width__param;
            pen.SendCustomEvent("_UpdateInkData");
            return;
        }

        public void __0__SetMeshLayer()
        {
            inkMeshLayer = __0_layer__param;
            pen.SendCustomEvent("_UpdateInkData");
            return;
        }

        public void __0__SetColliderLayer()
        {
            inkColliderLayer = __1_layer__param;
            pen.SendCustomEvent("_UpdateInkData");
            return;
        }

        public void __0__SetUsingDoubleClick()
        {
            pen.SetProgramVariable("__1_value__param", __0_value__param);
            pen.SendCustomEvent("__0__SetUseDoubleClick");
            return;
        }

        public void __0__SetEnabledLateSync()
        {
            pen.SetProgramVariable("__2_value__param", __1_value__param);
            pen.SendCustomEvent("__0__SetEnabledLateSync");
            return;
        }

        public void __0__SetUsingSurftraceMode()
        {
            pen.SetProgramVariable("__3_value__param", __2_value__param);
            pen.SendCustomEvent("__0__SetUseSurftraceMode");
            return;
        }

        public void ResetPen()
        {
            Clear();
            Respawn();
            return;
        }

        public void Respawn()
        {
            pen.SendCustomEvent("_Respawn");
            __0_isActive__param = true;
            function_0();
            return;
        }

        public void Clear()
        {
            _ClearSyncBuffer();
            pen.SendCustomEvent("_Clear");
            return;
        }

        public void UndoDraw()
        {
            if (pen.GetProgramVariable("isPickedUp"))
            {
                return;
            }
            else
            {
                _TakeOwnership();
                pen.SendCustomEvent("_UndoDraw");
                return;
            }
        }

        public void EraseOwnInk()
        {
            if (pen.GetProgramVariable("isPickedUp"))
            {
                return;
            }
            else
            {
                _TakeOwnership();
                pen.SendCustomEvent("_EraseOwnInk");
                return;
            }
        }

        public void __0_Register()
        {
            System.Boolean __intnl_SystemBoolean_29;
            __intnl_SystemBoolean_29 = !VRC.SDKBase.Utilities.IsValid(__0_listener__param);
            if (!__intnl_SystemBoolean_29)
            {
                __intnl_SystemBoolean_29 = listenerList.Contains((VRC.SDK3.Data.DataToken)__0_listener__param);
            }
            if (__intnl_SystemBoolean_29)
            {
                return;
            }
            else
            {
                listenerList.Add((VRC.SDK3.Data.DataToken)__0_listener__param);
                return;
            }
        }

        public void OnPenPickup()
        {
            VRC.Udon.UdonBehaviour __lcl_listener_VRCUdonUdonBehaviour_0;
            System.Int32 __lcl_i_SystemInt32_0;
            VRC.SDK3.Data.DataToken __lcl_listerToken_VRCSDK3DataDataToken_0;
            System.Int32 __lcl_n_SystemInt32_0;
            __lcl_i_SystemInt32_0 = 0;
            __lcl_n_SystemInt32_0 = listenerList.Count;
            while (__lcl_i_SystemInt32_0 < __lcl_n_SystemInt32_0)
            {
                if (listenerList.TryGetValue(__lcl_i_SystemInt32_0, null /* 15 */, out __lcl_listerToken_VRCSDK3DataDataToken_0))
                {
                    __lcl_listener_VRCUdonUdonBehaviour_0 = __lcl_listerToken_VRCSDK3DataDataToken_0.Reference;
                    if (VRC.SDKBase.Utilities.IsValid(__lcl_listener_VRCUdonUdonBehaviour_0))
                    {
                        __lcl_listener_VRCUdonUdonBehaviour_0.SendCustomEvent("OnPenPickup");
                    }
                }
                __lcl_i_SystemInt32_0 = __lcl_i_SystemInt32_0 + 1;
            }
            return;
        }

        public void OnPenDrop()
        {
            VRC.Udon.UdonBehaviour __lcl_listener_VRCUdonUdonBehaviour_1;
            VRC.SDK3.Data.DataToken __lcl_listerToken_VRCSDK3DataDataToken_1;
            System.Int32 __lcl_i_SystemInt32_1;
            System.Int32 __lcl_n_SystemInt32_1;
            __lcl_i_SystemInt32_1 = 0;
            __lcl_n_SystemInt32_1 = listenerList.Count;
            while (__lcl_i_SystemInt32_1 < __lcl_n_SystemInt32_1)
            {
                if (listenerList.TryGetValue(__lcl_i_SystemInt32_1, null /* 15 */, out __lcl_listerToken_VRCSDK3DataDataToken_1))
                {
                    __lcl_listener_VRCUdonUdonBehaviour_1 = __lcl_listerToken_VRCSDK3DataDataToken_1.Reference;
                    if (VRC.SDKBase.Utilities.IsValid(__lcl_listener_VRCUdonUdonBehaviour_1))
                    {
                        __lcl_listener_VRCUdonUdonBehaviour_1.SendCustomEvent("OnPenDrop");
                    }
                }
                __lcl_i_SystemInt32_1 = __lcl_i_SystemInt32_1 + 1;
            }
            return;
        }

        public void _TakeOwnership()
        {
            if (VRC.SDKBase.Networking.IsOwner(this.gameObject))
            {
                _ClearSyncBuffer();
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

        void get_isNetworkSettled()
        {
            System.Boolean __intnl_SystemBoolean_38;
            __intnl_SystemBoolean_38 = _isNetworkSettled;
            if (!__intnl_SystemBoolean_38)
            {
                _isNetworkSettled = VRC.SDKBase.Networking.IsNetworkSettled;
                __intnl_SystemBoolean_38 = _isNetworkSettled;
            }
            __0_get_isNetworkSettled__ret = __intnl_SystemBoolean_38;
            return;
        }

        void get_syncedData()
        {
            __0_get_syncedData__ret = _syncedData;
            return;
        }

        void function_2()
        {
            get_isNetworkSettled();
            if (__0_get_isNetworkSettled__ret)
            {
                _syncedData = __3_value__param;
                function_3();
                pen.SetProgramVariable("__5_data__param", _syncedData);
                pen.SetProgramVariable("__0_targetMode__param", 1);
                pen.SendCustomEvent("__0__UnpackData");
                return;
            }
            else
            {
                return;
            }
        }

        public void get_InkId()
        {
            __0_get_InkId__ret = inkId;
            return;
        }

        public void _IncrementInkId()
        {
            inkId = inkId + 1;
            return;
        }

        void function_3()
        {
            System.Boolean __intnl_SystemBoolean_39;
            System.Boolean __intnl_SystemBoolean_40;
            __intnl_SystemBoolean_40 = VRC.SDKBase.VRCPlayerApi.GetPlayerCount() > 1;
            if (__intnl_SystemBoolean_40)
            {
                __intnl_SystemBoolean_40 = VRC.SDKBase.Networking.IsOwner(this.gameObject);
            }
            __intnl_SystemBoolean_39 = __intnl_SystemBoolean_40;
            if (__intnl_SystemBoolean_39)
            {
                __intnl_SystemBoolean_39 = !isInUseSyncBuffer;
            }
            if (__intnl_SystemBoolean_39)
            {
                isInUseSyncBuffer = true;
                this.RequestSerialization();
            }
            return;
        }

        public void __0__SendData()
        {
            if (!isInUseSyncBuffer)
            {
                __3_value__param = __0_data__param;
                function_2();
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
            if (VRC.SDKBase.Networking.IsOwner(this.gameObject))
            {
                return;
            }
            else
            {
                __3_value__param = _syncedData;
                function_2();
                return;
            }
        }

        public void _onPostSerialization()
        {
            isInUseSyncBuffer = false;
            if (onPostSerializationResult.success)
            {
                pen.SetProgramVariable("__5_data__param", _syncedData);
                pen.SetProgramVariable("__0_targetMode__param", 1);
                pen.SendCustomEvent("__0__UnpackData");
            }
            else
            {
                pen.SetProgramVariable("__6_data__param", _syncedData);
                pen.SendCustomEvent("__0__EraseAbandonedInk");
            }
            return;
        }

        public void _ClearSyncBuffer()
        {
            __3_value__param = new UnityEngine.Vector3[](0);
            function_2();
            isInUseSyncBuffer = false;
            return;
        }
    }
}