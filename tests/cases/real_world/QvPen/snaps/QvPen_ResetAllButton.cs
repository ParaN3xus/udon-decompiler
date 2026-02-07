// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.UdonScript.UI
{
    public class QvPen_ResetAllButton : UdonSharpBehaviour
    {
        TMPro.TextMeshProUGUI messageTMPU = null;
        VRC.SDKBase.VRCPlayerApi master = null;
        VRC.SDKBase.VRCPlayerApi onOwnershipTransferredPlayer = null;
        VRC.SDKBase.VRCPlayerApi onPlayerJoinedPlayer = null;
        UnityEngine.UI.Text message = null;
        VRC.Udon.UdonBehaviour settings = null;
        TMPro.TextMeshPro messageTMP = null;

        public void _onPlayerJoined()
        {
            System.Boolean __intnl_SystemBoolean_0;
            __intnl_SystemBoolean_0 = master == null;
            if (!__intnl_SystemBoolean_0)
            {
                __intnl_SystemBoolean_0 = onPlayerJoinedPlayer.playerId < master.playerId;
            }
            if (__intnl_SystemBoolean_0)
            {
                master = onPlayerJoinedPlayer;
                function_0();
            }
            return;
        }

        public void _onOwnershipTransferred()
        {
            master = onOwnershipTransferredPlayer;
            function_0();
            return;
        }

        void function_0()
        {
            System.String __lcl_messageString_SystemString_0;
            System.String __lcl_s_SystemString_0;
            System.Int32 __lcl_i_SystemInt32_0;
            System.String __lcl_displayName_SystemString_0;
            System.Boolean __intnl_SystemBoolean_2;
            System.Boolean __intnl_SystemBoolean_3;
            System.Boolean __intnl_SystemBoolean_4;
            System.Int32 __lcl_cnt_SystemInt32_0;
            if (master == null)
            {
                return;
            }
            else
            {
                __lcl_displayName_SystemString_0 = System.String.Empty;
                __lcl_s_SystemString_0 = master.displayName;
                __lcl_cnt_SystemInt32_0 = 0;
                __lcl_i_SystemInt32_0 = 0;
                __intnl_SystemBoolean_2 = __lcl_i_SystemInt32_0 < __lcl_s_SystemString_0.Length;
                while (__intnl_SystemBoolean_2)
                {
                    __intnl_SystemBoolean_3 = System.Convert.ToInt32(__lcl_s_SystemString_0.ToCharArray(__lcl_i_SystemInt32_0, 1).Get(0)) < 128;
                    if (__intnl_SystemBoolean_3)
                    {
                        __lcl_cnt_SystemInt32_0 = __lcl_cnt_SystemInt32_0 + 1;
                    }
                    else
                    {
                        __lcl_cnt_SystemInt32_0 = __lcl_cnt_SystemInt32_0 + 2;
                    }
                    __intnl_SystemBoolean_4 = __lcl_cnt_SystemInt32_0 < 12;
                    if (!__intnl_SystemBoolean_4)
                    {
                        if (__lcl_i_SystemInt32_0 == __lcl_s_SystemString_0.Length - 1)
                        {
                            __lcl_displayName_SystemString_0 =
                                System.String.Concat(__lcl_displayName_SystemString_0, __lcl_s_SystemString_0.ToCharArray(__lcl_i_SystemInt32_0, 1).Get(0));
                            break;
                        }
                        else
                        {
                            __lcl_displayName_SystemString_0 = __lcl_displayName_SystemString_0 + "...";
                            break;
                        }
                    }
                    else
                    {
                        __lcl_displayName_SystemString_0 =
                            System.String.Concat(__lcl_displayName_SystemString_0, __lcl_s_SystemString_0.ToCharArray(__lcl_i_SystemInt32_0, 1).Get(0));
                        __lcl_i_SystemInt32_0 = __lcl_i_SystemInt32_0 + 1;
                    }
                    __intnl_SystemBoolean_2 = __lcl_i_SystemInt32_0 < __lcl_s_SystemString_0.Length;
                }
                __lcl_messageString_SystemString_0 = System.String.Format("<size=8>[Only {0}]</size>", __lcl_displayName_SystemString_0);
                __intnl_SystemBoolean_2 = (UnityEngine.Object)message;
                if (__intnl_SystemBoolean_2)
                {
                    message.text = __lcl_messageString_SystemString_0;
                }
                __intnl_SystemBoolean_3 = (UnityEngine.Object)messageTMP;
                if (__intnl_SystemBoolean_3)
                {
                    messageTMP.text = __lcl_messageString_SystemString_0;
                }
                __intnl_SystemBoolean_4 = (UnityEngine.Object)messageTMPU;
                if (__intnl_SystemBoolean_4)
                {
                    messageTMPU.text = __lcl_messageString_SystemString_0;
                }
                return;
            }
        }

        public void _interact()
        {
            System.Object __intnl_SystemObject_0;
            VRC.Udon.UdonBehaviour __lcl_penManager_VRCUdonUdonBehaviour_0;
            UnityEngine.Component[] __intnl_UnityEngineComponentArray_0;
            System.Int32 __intnl_SystemInt32_7;
            System.Int32 __intnl_SystemInt32_6;
            System.Boolean __intnl_SystemBoolean_8;
            System.Boolean __intnl_SystemBoolean_7;
            VRC.Udon.UdonBehaviour __lcl_eraserManager_VRCUdonUdonBehaviour_0;
            if (VRC.SDKBase.Networking.IsOwner(this.gameObject))
            {
                __intnl_SystemObject_0 = settings.GetProgramVariable("penManagers");
                __intnl_UnityEngineComponentArray_0 = __intnl_SystemObject_0;
                __intnl_SystemInt32_6 = __intnl_UnityEngineComponentArray_0.Length;
                __intnl_SystemInt32_7 = 0;
                __intnl_SystemBoolean_7 = __intnl_SystemInt32_7 < __intnl_SystemInt32_6;
                while (__intnl_SystemBoolean_7)
                {
                    __lcl_penManager_VRCUdonUdonBehaviour_0 = __intnl_UnityEngineComponentArray_0.Get(__intnl_SystemInt32_7);
                    __intnl_SystemBoolean_8 = (UnityEngine.Object)__lcl_penManager_VRCUdonUdonBehaviour_0;
                    if (__intnl_SystemBoolean_8)
                    {
                        __lcl_penManager_VRCUdonUdonBehaviour_0.SendCustomNetworkEvent(null /* 0 */, "ResetPen");
                    }
                    __intnl_SystemInt32_7 = __intnl_SystemInt32_7 + 1;
                    __intnl_SystemBoolean_7 = __intnl_SystemInt32_7 < __intnl_SystemInt32_6;
                }
                __intnl_SystemObject_0 = settings.GetProgramVariable("eraserManagers");
                __intnl_UnityEngineComponentArray_0 = __intnl_SystemObject_0;
                __intnl_SystemInt32_6 = __intnl_UnityEngineComponentArray_0.Length;
                __intnl_SystemInt32_7 = 0;
                __intnl_SystemBoolean_7 = __intnl_SystemInt32_7 < __intnl_SystemInt32_6;
                while (__intnl_SystemBoolean_7)
                {
                    __lcl_eraserManager_VRCUdonUdonBehaviour_0 = __intnl_UnityEngineComponentArray_0.Get(__intnl_SystemInt32_7);
                    __intnl_SystemBoolean_8 = (UnityEngine.Object)__lcl_eraserManager_VRCUdonUdonBehaviour_0;
                    if (__intnl_SystemBoolean_8)
                    {
                        __lcl_eraserManager_VRCUdonUdonBehaviour_0.SendCustomNetworkEvent(null /* 0 */, "ResetEraser");
                    }
                    __intnl_SystemInt32_7 = __intnl_SystemInt32_7 + 1;
                    __intnl_SystemBoolean_7 = __intnl_SystemInt32_7 < __intnl_SystemInt32_6;
                }
                return;
            }
            else
            {
                return;
            }
        }
    }
}