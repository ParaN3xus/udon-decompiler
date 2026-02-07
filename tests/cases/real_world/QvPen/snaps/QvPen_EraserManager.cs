// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.UdonScript
{
    public class QvPen_EraserManager : UdonSharpBehaviour
    {
        System.Boolean _isNetworkSettled = false;
        UnityEngine.Vector3[] __0_value__param = null;
        UnityEngine.GameObject respawnButton = null;
        VRC.Udon.Common.SerializationResult onPostSerializationResult = null /* {"success": false, "byteCount": 0} */;
        System.Int32 inkColliderLayer = 9;
        UnityEngine.UI.Text textInUse = null;
        UnityEngine.Vector3[] __0_data__param = null;
        VRC.Udon.UdonBehaviour eraser = null;
        TMPro.TextMeshProUGUI textInUseTMPU = null;
        VRC.SDKBase.VRCPlayerApi onPlayerJoinedPlayer = null;
        TMPro.TextMeshPro textInUseTMP = null;
        System.Boolean isInUseSyncBuffer = false;
        VRC.SDKBase.VRCPlayerApi onPlayerLeftPlayer = null;
        UnityEngine.Vector3[] __0_get_syncedData__ret = null;
        UnityEngine.GameObject inUseUI = null;
        System.Boolean __0__TakeOwnership__ret = false;
        UnityEngine.Vector3[] _syncedData = null;
        System.Boolean __0_get_isNetworkSettled__ret = false;

        public void _start()
        {
            eraser.SetProgramVariable("__0_eraserManager__param", this);
            eraser.SendCustomEvent("__0__Init");
            return;
        }

        public void _onPlayerJoined()
        {
            System.Boolean __intnl_SystemBoolean_0;
            __intnl_SystemBoolean_0 = VRC.SDKBase.Networking.LocalPlayer.IsOwner(eraser.gameObject);
            if (__intnl_SystemBoolean_0)
            {
                eraser.SendCustomEvent("get_IsUser");
                __intnl_SystemBoolean_0 = eraser.GetProgramVariable("__0_get_IsUser__ret");
            }
            if (__intnl_SystemBoolean_0)
            {
                this.SendCustomNetworkEvent(null /* 0 */, "StartUsing");
            }
            return;
        }

        public void _onPlayerLeft()
        {
            System.Boolean __intnl_SystemBoolean_2;
            __intnl_SystemBoolean_2 = VRC.SDKBase.Networking.IsOwner(eraser.gameObject);
            if (__intnl_SystemBoolean_2)
            {
                eraser.SendCustomEvent("get_IsUser");
                __intnl_SystemBoolean_2 = !eraser.GetProgramVariable("__0_get_IsUser__ret");
            }
            if (__intnl_SystemBoolean_2)
            {
                eraser.SendCustomEvent("_onDrop");
            }
            return;
        }

        public void StartUsing()
        {
            System.String __lcl_text_SystemString_0;
            VRC.SDKBase.VRCPlayerApi __lcl_owner_VRCSDKBaseVRCPlayerApi_0;
            eraser.SetProgramVariable("isPickedUp", true);
            respawnButton.SetActive(false);
            inUseUI.SetActive(true);
            __lcl_owner_VRCSDKBaseVRCPlayerApi_0 = VRC.SDKBase.Networking.GetOwner(eraser.gameObject);
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
            eraser.SetProgramVariable("isPickedUp", false);
            respawnButton.SetActive(true);
            inUseUI.SetActive(false);
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

        public void ResetEraser()
        {
            eraser.SendCustomEvent("_Respawn");
            return;
        }

        public void Respawn()
        {
            eraser.SendCustomEvent("_Respawn");
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
            System.Boolean __intnl_SystemBoolean_12;
            __intnl_SystemBoolean_12 = _isNetworkSettled;
            if (!__intnl_SystemBoolean_12)
            {
                _isNetworkSettled = VRC.SDKBase.Networking.IsNetworkSettled;
                __intnl_SystemBoolean_12 = _isNetworkSettled;
            }
            __0_get_isNetworkSettled__ret = __intnl_SystemBoolean_12;
            return;
        }

        void get_syncedData()
        {
            __0_get_syncedData__ret = _syncedData;
            return;
        }

        void function_0()
        {
            get_isNetworkSettled();
            if (__0_get_isNetworkSettled__ret)
            {
                _syncedData = __0_value__param;
                function_1();
                eraser.SetProgramVariable("__4_data__param", _syncedData);
                eraser.SendCustomEvent("__0__UnpackData");
                return;
            }
            else
            {
                return;
            }
        }

        void function_1()
        {
            System.Boolean __intnl_SystemBoolean_13;
            System.Boolean __intnl_SystemBoolean_14;
            __intnl_SystemBoolean_14 = VRC.SDKBase.VRCPlayerApi.GetPlayerCount() > 1;
            if (__intnl_SystemBoolean_14)
            {
                __intnl_SystemBoolean_14 = VRC.SDKBase.Networking.IsOwner(this.gameObject);
            }
            __intnl_SystemBoolean_13 = __intnl_SystemBoolean_14;
            if (__intnl_SystemBoolean_13)
            {
                __intnl_SystemBoolean_13 = !isInUseSyncBuffer;
            }
            if (__intnl_SystemBoolean_13)
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
                __0_value__param = __0_data__param;
                function_0();
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
            __0_value__param = _syncedData;
            function_0();
            return;
        }

        public void _onPostSerialization()
        {
            isInUseSyncBuffer = false;
            if (onPostSerializationResult.success)
            {
                eraser.SendCustomEvent("ExecuteEraseInk");
            }
            return;
        }

        public void _ClearSyncBuffer()
        {
            __0_value__param = new UnityEngine.Vector3[](0);
            function_0();
            isInUseSyncBuffer = false;
            return;
        }
    }
}