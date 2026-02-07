<!-- ci: skip-compile -->

```csharp
using TMPro;
using UdonSharp;
using UnityEngine;
using UnityEngine.UI;
using VRC.SDKBase;
using VRC.Udon.Common.Interfaces;

namespace QvPen.UdonScript.UI
{
    [AddComponentMenu("")]
    [UdonBehaviourSyncMode(BehaviourSyncMode.None)]
    public class QvPen_ResetAllButton : UdonSharpBehaviour
    {
        [SerializeField]
        private QvPen_Settings settings;

        [SerializeField]
        private Text message;
        [SerializeField]
        private TextMeshPro messageTMP;
        [SerializeField]
        private TextMeshProUGUI messageTMPU;

        private VRCPlayerApi master = null;

        public override void OnPlayerJoined(VRCPlayerApi player)
        {
            if (master == null || player.playerId < master.playerId)
            {
                master = player;
                UpdateMessage();
            }
        }

        public override void OnOwnershipTransferred(VRCPlayerApi player)
        {
            master = player;
            UpdateMessage();
        }

        private void UpdateMessage()
        {
            if (master == null)
                return;

            var displayName = string.Empty;

            var s = master.displayName;
            var cnt = 0;
            for (var i = 0; i < s.Length; i++)
            {
                if (s[i] < 128)
                    cnt += 1;
                else
                    cnt += 2;

                if (cnt < 12)
                    displayName += s[i];
                else
                {
                    if (i == s.Length - 1)
                        displayName += s[i];
                    else
                        displayName += "...";
                    break;
                }
            }

            var messageString = $"<size=8>[Only {displayName}]</size>";

            if (message)
                message.text = messageString;

            if (messageTMP)
                messageTMP.text = messageString;

            if (messageTMPU)
                messageTMPU.text = messageString;
        }

        public override void Interact()
        {
            if (!Networking.IsOwner(gameObject))
                return;

            foreach (var penManager in settings.penManagers)
                if (penManager)
                    penManager.SendCustomNetworkEvent(NetworkEventTarget.All, nameof(QvPen_PenManager.ResetPen));

            foreach (var eraserManager in settings.eraserManagers)
                if (eraserManager)
                    eraserManager.SendCustomNetworkEvent(NetworkEventTarget.All, nameof(QvPen_EraserManager.ResetEraser));
        }
    }
}
```

```json
{
  "byteCodeHex": "00000001000000090000000100000007000000010000000A000000010000001B000000060000003B000000010000001B000000040000004000000005000000900000000100000008000000010000001C000000060000003C0000000100000007000000010000001D000000060000003C000000010000001C000000010000001D000000010000001B000000060000003D000000010000001B00000004000000C40000000100000008000000010000000700000009000000010000000B000000050000012000000001000000020000000900000008000000020000000100000009000000010000000C000000010000000700000009000000010000000D0000000500000120000000010000000200000009000000080000000200000001000000090000000100000007000000010000000A000000010000001E000000060000003B000000010000001E00000004000001640000000100000002000000090000000800000002000000010000001F000000060000003E00000001000000070000000100000020000000060000003F000000010000000E000000010000002100000009000000010000000E000000010000002600000009000000010000002000000001000000270000000600000040000000010000002600000001000000270000000100000023000000060000003D0000000100000023000000040000049C00000001000000200000000100000026000000010000000F000000010000002900000006000000410000000100000029000000010000000E000000010000002800000006000000420000000100000028000000010000002A0000000600000043000000010000002A00000001000000100000000100000024000000060000003D000000010000002400000004000002B40000000100000021000000010000000F0000000100000021000000060000004400000005000002D40000000100000021000000010000001100000001000000210000000600000044000000010000002100000001000000120000000100000025000000060000003D0000000100000025000000040000037400000001000000200000000100000026000000010000000F000000010000002C0000000600000041000000010000002C000000010000000E000000010000002B0000000600000042000000010000001F000000010000002B000000010000001F000000060000004500000005000004740000000100000020000000010000002D0000000600000040000000010000002D000000010000000F000000010000002E00000006000000460000000100000026000000010000002E000000010000002F0000000600000047000000010000002F000000040000044C00000001000000200000000100000026000000010000000F000000010000003100000006000000410000000100000031000000010000000E00000001000000300000000600000042000000010000001F0000000100000030000000010000001F0000000600000045000000050000046C000000010000001F0000000100000013000000010000001F0000000600000048000000050000049C0000000100000026000000010000000F0000000100000026000000060000004400000005000001B40000000100000014000000010000001F0000000100000022000000060000004900000001000000040000000100000023000000060000004A000000010000002300000004000004FC00000001000000040000000100000022000000060000004B00000001000000050000000100000024000000060000004A0000000100000024000000040000053C00000001000000050000000100000022000000060000004C00000001000000060000000100000025000000060000004A0000000100000025000000040000057C00000001000000060000000100000022000000060000004D0000000100000002000000090000000800000002000000010000000900000001000000150000000100000032000000060000004E000000010000003200000004000005C800000005000005DC0000000100000002000000090000000800000002000000010000000300000001000000160000000100000033000000060000004F0000000100000033000000010000003400000009000000010000003400000001000000350000000600000050000000010000000E000000010000003600000009000000010000003600000001000000350000000100000037000000060000003D000000010000003700000004000006FC000000010000003400000001000000360000000100000038000000060000005100000001000000380000000100000039000000060000004A000000010000003900000004000006D400000001000000380000000100000017000000010000001800000006000000520000000100000036000000010000000F00000001000000360000000600000044000000050000063C000000010000000300000001000000190000000100000033000000060000004F0000000100000033000000010000003400000009000000010000003400000001000000350000000600000050000000010000000E000000010000003600000009000000010000003600000001000000350000000100000037000000060000003D0000000100000037000000040000081C00000001000000340000000100000036000000010000003A0000000600000051000000010000003A0000000100000039000000060000004A000000010000003900000004000007F4000000010000003A0000000100000017000000010000001A00000006000000520000000100000036000000010000000F00000001000000360000000600000044000000050000075C0000000100000002000000090000000800000002",
  "byteCodeLength": 2096,
  "symbols": {
    "messageTMPU": {
      "name": "messageTMPU",
      "type": "TMPro.TextMeshProUGUI",
      "address": 6
    },
    "__intnl_SystemObject_0": {
      "name": "__intnl_SystemObject_0",
      "type": "System.Object",
      "address": 51
    },
    "__intnl_SystemChar_1": {
      "name": "__intnl_SystemChar_1",
      "type": "System.Char",
      "address": 43
    },
    "__const_SystemString_0": {
      "name": "__const_SystemString_0",
      "type": "System.String",
      "address": 19
    },
    "__gintnl_SystemUInt32_1": {
      "name": "__gintnl_SystemUInt32_1",
      "type": "System.UInt32",
      "address": 13
    },
    "__gintnl_SystemUInt32_0": {
      "name": "__gintnl_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 11
    },
    "__lcl_penManager_VRCUdonUdonBehaviour_0": {
      "name": "__lcl_penManager_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 56
    },
    "__const_SystemString_5": {
      "name": "__const_SystemString_5",
      "type": "System.String",
      "address": 26
    },
    "__const_VRCUdonCommonInterfacesNetworkEventTarget_0": {
      "name": "__const_VRCUdonCommonInterfacesNetworkEventTarget_0",
      "type": "VRC.Udon.Common.Interfaces.NetworkEventTarget",
      "address": 23
    },
    "__intnl_SystemCharArray_2": {
      "name": "__intnl_SystemCharArray_2",
      "type": "System.Char[]",
      "address": 49
    },
    "__intnl_SystemCharArray_0": {
      "name": "__intnl_SystemCharArray_0",
      "type": "System.Char[]",
      "address": 41
    },
    "__intnl_SystemCharArray_1": {
      "name": "__intnl_SystemCharArray_1",
      "type": "System.Char[]",
      "address": 44
    },
    "__refl_typeid": {
      "name": "__refl_typeid",
      "type": "System.Int64",
      "address": 0
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 9
    },
    "__const_SystemString_2": {
      "name": "__const_SystemString_2",
      "type": "System.String",
      "address": 22
    },
    "__intnl_UnityEngineComponentArray_0": {
      "name": "__intnl_UnityEngineComponentArray_0",
      "type": "UnityEngine.Component[]",
      "address": 52
    },
    "__const_SystemObject_0": {
      "name": "__const_SystemObject_0",
      "type": "System.Object",
      "address": 10
    },
    "__intnl_SystemChar_0": {
      "name": "__intnl_SystemChar_0",
      "type": "System.Char",
      "address": 40
    },
    "__lcl_messageString_SystemString_0": {
      "name": "__lcl_messageString_SystemString_0",
      "type": "System.String",
      "address": 34
    },
    "master": {
      "name": "master",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 7
    },
    "__lcl_s_SystemString_0": {
      "name": "__lcl_s_SystemString_0",
      "type": "System.String",
      "address": 32
    },
    "onOwnershipTransferredPlayer": {
      "name": "onOwnershipTransferredPlayer",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 12
    },
    "onPlayerJoinedPlayer": {
      "name": "onPlayerJoinedPlayer",
      "type": "VRC.SDKBase.VRCPlayerApi",
      "address": 8
    },
    "__intnl_returnJump_SystemUInt32_0": {
      "name": "__intnl_returnJump_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 2
    },
    "__const_SystemString_4": {
      "name": "__const_SystemString_4",
      "type": "System.String",
      "address": 25
    },
    "__lcl_i_SystemInt32_0": {
      "name": "__lcl_i_SystemInt32_0",
      "type": "System.Int32",
      "address": 38
    },
    "message": {
      "name": "message",
      "type": "UnityEngine.UI.Text",
      "address": 4
    },
    "__refl_typename": {
      "name": "__refl_typename",
      "type": "System.String",
      "address": 1
    },
    "settings": {
      "name": "settings",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 3
    },
    "__intnl_SystemInt32_1": {
      "name": "__intnl_SystemInt32_1",
      "type": "System.Int32",
      "address": 29
    },
    "__intnl_SystemInt32_0": {
      "name": "__intnl_SystemInt32_0",
      "type": "System.Int32",
      "address": 28
    },
    "__intnl_SystemInt32_3": {
      "name": "__intnl_SystemInt32_3",
      "type": "System.Int32",
      "address": 42
    },
    "__intnl_SystemInt32_2": {
      "name": "__intnl_SystemInt32_2",
      "type": "System.Int32",
      "address": 39
    },
    "__intnl_SystemInt32_5": {
      "name": "__intnl_SystemInt32_5",
      "type": "System.Int32",
      "address": 46
    },
    "__intnl_SystemInt32_4": {
      "name": "__intnl_SystemInt32_4",
      "type": "System.Int32",
      "address": 45
    },
    "__intnl_SystemInt32_7": {
      "name": "__intnl_SystemInt32_7",
      "type": "System.Int32",
      "address": 54
    },
    "__intnl_SystemInt32_6": {
      "name": "__intnl_SystemInt32_6",
      "type": "System.Int32",
      "address": 53
    },
    "__intnl_SystemChar_2": {
      "name": "__intnl_SystemChar_2",
      "type": "System.Char",
      "address": 48
    },
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 20
    },
    "messageTMP": {
      "name": "messageTMP",
      "type": "TMPro.TextMeshPro",
      "address": 5
    },
    "__this_UnityEngineGameObject_0": {
      "name": "__this_UnityEngineGameObject_0",
      "type": "UnityEngine.GameObject",
      "address": 21
    },
    "__lcl_displayName_SystemString_0": {
      "name": "__lcl_displayName_SystemString_0",
      "type": "System.String",
      "address": 31
    },
    "__const_SystemInt32_1": {
      "name": "__const_SystemInt32_1",
      "type": "System.Int32",
      "address": 15
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 14
    },
    "__const_SystemInt32_3": {
      "name": "__const_SystemInt32_3",
      "type": "System.Int32",
      "address": 17
    },
    "__const_SystemInt32_2": {
      "name": "__const_SystemInt32_2",
      "type": "System.Int32",
      "address": 16
    },
    "__const_SystemInt32_4": {
      "name": "__const_SystemInt32_4",
      "type": "System.Int32",
      "address": 18
    },
    "__intnl_SystemBoolean_8": {
      "name": "__intnl_SystemBoolean_8",
      "type": "System.Boolean",
      "address": 57
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 27
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 30
    },
    "__intnl_SystemBoolean_2": {
      "name": "__intnl_SystemBoolean_2",
      "type": "System.Boolean",
      "address": 35
    },
    "__intnl_SystemBoolean_3": {
      "name": "__intnl_SystemBoolean_3",
      "type": "System.Boolean",
      "address": 36
    },
    "__intnl_SystemBoolean_4": {
      "name": "__intnl_SystemBoolean_4",
      "type": "System.Boolean",
      "address": 37
    },
    "__intnl_SystemBoolean_5": {
      "name": "__intnl_SystemBoolean_5",
      "type": "System.Boolean",
      "address": 47
    },
    "__intnl_SystemBoolean_6": {
      "name": "__intnl_SystemBoolean_6",
      "type": "System.Boolean",
      "address": 50
    },
    "__intnl_SystemBoolean_7": {
      "name": "__intnl_SystemBoolean_7",
      "type": "System.Boolean",
      "address": 55
    },
    "__lcl_cnt_SystemInt32_0": {
      "name": "__lcl_cnt_SystemInt32_0",
      "type": "System.Int32",
      "address": 33
    },
    "__lcl_eraserManager_VRCUdonUdonBehaviour_0": {
      "name": "__lcl_eraserManager_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 58
    },
    "__const_SystemString_3": {
      "name": "__const_SystemString_3",
      "type": "System.String",
      "address": 24
    }
  },
  "entryPoints": [
    {
      "name": "_onPlayerJoined",
      "address": 0
    },
    {
      "name": "_onOwnershipTransferred",
      "address": 216
    },
    {
      "name": "_interact",
      "address": 1424
    }
  ],
  "heapInitialValues": {
    "0": {
      "address": 0,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": -6765922199923257912
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen.UdonScript.UI.QvPen_ResetAllButton"
      }
    },
    "2": {
      "address": 2,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "3": {
      "address": 3,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "4": {
      "address": 4,
      "type": "UnityEngine.UI.Text",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "5": {
      "address": 5,
      "type": "TMPro.TextMeshPro",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "6": {
      "address": 6,
      "type": "TMPro.TextMeshProUGUI",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "7": {
      "address": 7,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "8": {
      "address": 8,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "9": {
      "address": 9,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
      }
    },
    "10": {
      "address": 10,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "11": {
      "address": 11,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 196
      }
    },
    "12": {
      "address": 12,
      "type": "VRC.SDKBase.VRCPlayerApi",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "13": {
      "address": 13,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 260
      }
    },
    "14": {
      "address": 14,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "15": {
      "address": 15,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 1
      }
    },
    "16": {
      "address": 16,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 128
      }
    },
    "17": {
      "address": 17,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 2
      }
    },
    "18": {
      "address": 18,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 12
      }
    },
    "19": {
      "address": 19,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "..."
      }
    },
    "20": {
      "address": 20,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "<size=8>[Only {0}]</size>"
      }
    },
    "21": {
      "address": 21,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "UnityEngine.GameObject, UnityEngine.CoreModule, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "22": {
      "address": 22,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "penManagers"
      }
    },
    "23": {
      "address": 23,
      "type": "VRC.Udon.Common.Interfaces.NetworkEventTarget",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "24": {
      "address": 24,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "ResetPen"
      }
    },
    "25": {
      "address": 25,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "eraserManagers"
      }
    },
    "26": {
      "address": 26,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "ResetEraser"
      }
    },
    "27": {
      "address": 27,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "28": {
      "address": 28,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "29": {
      "address": 29,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "30": {
      "address": 30,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "31": {
      "address": 31,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "32": {
      "address": 32,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "33": {
      "address": 33,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "34": {
      "address": 34,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "35": {
      "address": 35,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "36": {
      "address": 36,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "37": {
      "address": 37,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "38": {
      "address": 38,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "39": {
      "address": 39,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "40": {
      "address": 40,
      "type": "System.Char",
      "value": {
        "isSerializable": true,
        "value": "\u0000"
      }
    },
    "41": {
      "address": 41,
      "type": "System.Char[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "42": {
      "address": 42,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "43": {
      "address": 43,
      "type": "System.Char",
      "value": {
        "isSerializable": true,
        "value": "\u0000"
      }
    },
    "44": {
      "address": 44,
      "type": "System.Char[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "45": {
      "address": 45,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "46": {
      "address": 46,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "47": {
      "address": 47,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "48": {
      "address": 48,
      "type": "System.Char",
      "value": {
        "isSerializable": true,
        "value": "\u0000"
      }
    },
    "49": {
      "address": 49,
      "type": "System.Char[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "50": {
      "address": 50,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "51": {
      "address": 51,
      "type": "System.Object",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "52": {
      "address": 52,
      "type": "UnityEngine.Component[]",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "53": {
      "address": 53,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "54": {
      "address": 54,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "55": {
      "address": 55,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "56": {
      "address": 56,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "57": {
      "address": 57,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "58": {
      "address": 58,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "59": {
      "address": 59,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObject.__op_Equality__SystemObject_SystemObject__SystemBoolean"
      }
    },
    "60": {
      "address": 60,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__get_playerId__SystemInt32"
      }
    },
    "61": {
      "address": 61,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_LessThan__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "62": {
      "address": 62,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__get_Empty__SystemString"
      }
    },
    "63": {
      "address": 63,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseVRCPlayerApi.__get_displayName__SystemString"
      }
    },
    "64": {
      "address": 64,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__get_Length__SystemInt32"
      }
    },
    "65": {
      "address": 65,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__ToCharArray__SystemInt32_SystemInt32__SystemCharArray"
      }
    },
    "66": {
      "address": 66,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemCharArray.__Get__SystemInt32__SystemChar"
      }
    },
    "67": {
      "address": 67,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemConvert.__ToInt32__SystemChar__SystemInt32"
      }
    },
    "68": {
      "address": 68,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Addition__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "69": {
      "address": 69,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Concat__SystemObject_SystemObject__SystemString"
      }
    },
    "70": {
      "address": 70,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Subtraction__SystemInt32_SystemInt32__SystemInt32"
      }
    },
    "71": {
      "address": 71,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemInt32.__op_Equality__SystemInt32_SystemInt32__SystemBoolean"
      }
    },
    "72": {
      "address": 72,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__op_Addition__SystemString_SystemString__SystemString"
      }
    },
    "73": {
      "address": 73,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemString.__Format__SystemString_SystemObject__SystemString"
      }
    },
    "74": {
      "address": 74,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineObject.__op_Implicit__UnityEngineObject__SystemBoolean"
      }
    },
    "75": {
      "address": 75,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineUIText.__set_text__SystemString__SystemVoid"
      }
    },
    "76": {
      "address": 76,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "TMProTextMeshPro.__set_text__SystemString__SystemVoid"
      }
    },
    "77": {
      "address": 77,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "TMProTextMeshProUGUI.__set_text__SystemString__SystemVoid"
      }
    },
    "78": {
      "address": 78,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseNetworking.__IsOwner__UnityEngineGameObject__SystemBoolean"
      }
    },
    "79": {
      "address": 79,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__GetProgramVariable__SystemString__SystemObject"
      }
    },
    "80": {
      "address": 80,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemArray.__get_Length__SystemInt32"
      }
    },
    "81": {
      "address": 81,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemObjectArray.__Get__SystemInt32__SystemObject"
      }
    },
    "82": {
      "address": 82,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomNetworkEvent__VRCUdonCommonInterfacesNetworkEventTarget_SystemString__SystemVoid"
      }
    }
  }
}
```