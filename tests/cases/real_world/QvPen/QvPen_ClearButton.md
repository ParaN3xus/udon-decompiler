<!-- ci: skip-compile -->

```csharp
﻿using UdonSharp;
using UnityEngine;
using UnityEngine.UI;
using VRC.Udon.Common;
using VRC.Udon.Common.Interfaces;
using Utilities = VRC.SDKBase.Utilities;

namespace QvPen.Udon.UI
{
    using QvPen.UdonScript;

    [AddComponentMenu("")]
    [UdonBehaviourSyncMode(BehaviourSyncMode.None)]
    public class QvPen_ClearButton : QvPen_PenCallbackListener
    {
        [SerializeField]
        private QvPen_PenManager penManager;

        [SerializeField]
        private Image ownTextImage;

        [SerializeField]
        private Image ownIndicator;

        [SerializeField]
        private Image allTextImage;

        [SerializeField]
        private Image allIndicator;

        private const float keepSeconds1 = 0.31f;
        private const float keepSeconds2 = 2f;

        private float targetTime1;
        private float targetTime2;

        private bool isInInteract;

        private bool isPickedUp;
        private const float keepSeconds_TextImage = 5f;
        private float targetTime_TextImage;

        private void Start()
        {
            SetActiveIndicator(false);
            SetActiveTextImage(false);

            penManager.Register(this);
        }

        public override void InputUse(bool value, UdonInputEventArgs args)
        {
            if (value)
                return;

            if (isInInteract && Time.time < targetTime1)
                UndoDraw();

            isInInteract = false;
            SetActiveIndicator(false);
        }

        public override void Interact()
        {
            isInInteract = true;
            SetActiveIndicator(true);

            targetTime1 = Time.time + keepSeconds1;
            targetTime2 = Time.time + keepSeconds2;
            targetTime_TextImage = Time.time + keepSeconds_TextImage;

            SendCustomEventDelayedSeconds(nameof(_LoopIndicator1), 0f);
            Enter_LoopTextImageActive();
        }

        public override void OnPenPickup()
        {
            isPickedUp = true;
            Enter_LoopTextImageActive();
        }

        public override void OnPenDrop()
        {
            isPickedUp = false;

            targetTime_TextImage = Time.time + keepSeconds_TextImage;
            Enter_LoopTextImageActive();
        }

        private void SetActiveIndicator(bool isActive)
        {
            if (Utilities.IsValid(ownIndicator))
            {
                ownIndicator.gameObject.SetActive(isActive);
                ownIndicator.fillAmount = 0f;
            }

            if (Utilities.IsValid(allIndicator))
            {
                allIndicator.gameObject.SetActive(isActive);
                allIndicator.fillAmount = 0f;
            }
        }

        private void SetActiveTextImage(bool isActive)
        {
            if (Utilities.IsValid(ownTextImage))
                ownTextImage.gameObject.SetActive(isActive);

            if (Utilities.IsValid(allTextImage))
                allTextImage.gameObject.SetActive(isActive);
        }

        private void SeValueIndicator(float ownIndicatorValue, float allIndicatorValue)
        {
            if (Utilities.IsValid(ownIndicator))
                ownIndicator.fillAmount = Mathf.Clamp01(ownIndicatorValue);

            if (Utilities.IsValid(allIndicator))
                allIndicator.fillAmount = Mathf.Clamp01(allIndicatorValue);
        }

        public void _LoopIndicator1()
        {
            if (!isInInteract)
                return;

            var time = Time.time;

            var leaveTime1 = targetTime1 - time;
            var leaveTime2 = targetTime2 - time;
            if (leaveTime1 <= 0f)
            {
                EraseOwnInk();

                SeValueIndicator(1f, 1f - leaveTime2 / keepSeconds2);

                SendCustomEventDelayedFrames(nameof(_LoopIndicator2), 0);

                return;
            }

            SeValueIndicator(1f - leaveTime1 / keepSeconds1, 1f - leaveTime2 / keepSeconds2);

            SendCustomEventDelayedFrames(nameof(_LoopIndicator1), 0);
        }

        public void _LoopIndicator2()
        {
            if (!isInInteract)
                return;

            var leaveTime2 = targetTime2 - Time.time;
            if (leaveTime2 <= 0f)
            {
                Clear();

                SeValueIndicator(0f, 0f);

                return;
            }

            SeValueIndicator(0f, 1f - leaveTime2 / keepSeconds2);

            SendCustomEventDelayedFrames(nameof(_LoopIndicator2), 0);
        }

        private bool isIn_LoopTextImageActive;

        private void Enter_LoopTextImageActive()
        {
            if (isIn_LoopTextImageActive)
                return;

            isIn_LoopTextImageActive = true;

            if (isPickedUp)
            {
                Exit_LoopTextImageActive();
                return;
            }

            SetActiveTextImage(true);

            _LoopTextImageActive();
        }

        private void Exit_LoopTextImageActive()
        {
            if (!isIn_LoopTextImageActive)
                return;

            isIn_LoopTextImageActive = false;

            SetActiveTextImage(false);
        }

        public void _LoopTextImageActive()
        {
            if (!isIn_LoopTextImageActive)
                return;

            if (isPickedUp)
            {
                Exit_LoopTextImageActive();
                return;
            }

            var time = Time.time;

            var leaveTime_TextImage = targetTime_TextImage - time;
            if (leaveTime_TextImage <= 0f)
            {
                Exit_LoopTextImageActive();
                return;
            }

            SendCustomEventDelayedSeconds(nameof(_LoopTextImageActive), leaveTime_TextImage / 2f);
        }

        private void EraseOwnInk()
        {
            if (Utilities.IsValid(penManager))
                penManager.EraseOwnInk();
        }

        private void UndoDraw()
        {
            if (Utilities.IsValid(penManager))
                penManager.UndoDraw();
        }

        private void Clear()
        {
            if (Utilities.IsValid(penManager))
                penManager.SendCustomNetworkEvent(NetworkEventTarget.All, nameof(QvPen_PenManager.Clear));
        }
    }
}
```

```json
{
  "byteCodeHex": "000000010000000F0000000100000010000000010000000C0000000900000001000000110000000500000A140000000100000003000000090000000800000003000000010000000F0000000100000012000000010000000C000000090000000100000042000000060000006A00000001000000420000000100000013000000010000000D000000060000006B00000001000000140000000500000A140000000100000003000000090000000800000003000000010000000F00000001000000150000000100000012000000010000001600000009000000050000037000000001000000170000000100000012000000010000001800000009000000050000046C00000001000000190000000100000043000000090000000100000004000000010000001A0000000100000043000000060000006C0000000100000004000000010000001B000000060000006D0000000100000003000000090000000800000003000000010000000F000000010000001C000000040000018C0000000100000003000000090000000800000003000000010000000B000000010000004400000009000000010000004400000004000001E00000000100000045000000060000006A000000010000004500000001000000090000000100000044000000060000006E00000001000000440000000400000200000000010000001E0000000500000D080000000100000012000000010000000B00000009000000010000001F000000010000001200000001000000160000000900000005000003700000000100000003000000090000000800000003000000010000000F0000000100000010000000010000000B000000090000000100000020000000010000001000000001000000160000000900000005000003700000000100000046000000060000006A000000010000004600000001000000210000000100000009000000060000006B0000000100000047000000060000006A00000001000000470000000100000022000000010000000A000000060000006B0000000100000048000000060000006A00000001000000480000000100000013000000010000000D000000060000006B0000000100000023000000010000002400000001000000250000000100000026000000060000006F00000001000000270000000500000A140000000100000003000000090000000800000003000000010000000F000000010000000600000001000000490000000600000070000000010000004900000004000003E00000000100000006000000010000004B0000000600000071000000010000004B000000010000001600000006000000720000000100000006000000010000002500000006000000730000000100000008000000010000004A0000000600000070000000010000004A00000004000004500000000100000008000000010000004B0000000600000071000000010000004B000000010000001600000006000000720000000100000008000000010000002500000006000000730000000100000003000000090000000800000003000000010000000F0000000100000005000000010000004C0000000600000070000000010000004C00000004000004C40000000100000005000000010000004D0000000600000071000000010000004D000000010000001800000006000000720000000100000007000000010000004E0000000600000070000000010000004E000000040000051C0000000100000007000000010000004F0000000600000071000000010000004F000000010000001800000006000000720000000100000003000000090000000800000003000000010000000F00000001000000060000000100000050000000060000007000000001000000500000000400000590000000010000002800000001000000510000000600000074000000010000000600000001000000510000000600000073000000010000000800000001000000520000000600000070000000010000005200000004000005E80000000100000029000000010000005300000006000000740000000100000008000000010000005300000006000000730000000100000003000000090000000800000003000000010000000F000000010000000B000000040000061C000000050000063000000001000000030000000900000008000000030000000100000054000000060000006A0000000100000009000000010000005400000001000000550000000600000075000000010000000A000000010000005400000001000000560000000600000075000000010000005500000001000000250000000100000057000000060000007600000001000000570000000400000774000000010000002A0000000500000C98000000010000002B0000000100000056000000010000002200000001000000580000000600000077000000010000002C000000010000005800000001000000590000000600000075000000010000002C00000001000000280000000900000001000000590000000100000029000000090000000500000538000000010000002D000000010000002E000000010000002F00000001000000260000000600000078000000010000000300000009000000080000000300000001000000300000000100000055000000010000002100000001000000580000000600000077000000010000002C00000001000000580000000100000059000000060000007500000001000000560000000100000022000000010000005A0000000600000077000000010000002C000000010000005A000000010000005B00000006000000750000000100000059000000010000002800000009000000010000005B000000010000002900000009000000050000053800000001000000310000000100000024000000010000002F000000010000002600000006000000780000000100000003000000090000000800000003000000010000000F000000010000000B0000000400000888000000050000089C0000000100000003000000090000000800000003000000010000005D000000060000006A000000010000000A000000010000005D000000010000005C0000000600000075000000010000005C0000000100000025000000010000005E0000000600000076000000010000005E000000040000095800000001000000320000000500000D78000000010000003300000001000000250000000100000028000000090000000100000025000000010000002900000009000000050000053800000001000000030000000900000008000000030000000100000034000000010000005C0000000100000022000000010000005F0000000600000077000000010000002C000000010000005F000000010000006000000006000000750000000100000025000000010000002800000009000000010000006000000001000000290000000900000005000005380000000100000035000000010000002E000000010000002F000000010000002600000006000000780000000100000003000000090000000800000003000000010000000F000000010000000E0000000400000A3800000001000000030000000900000008000000030000000100000010000000010000000E00000009000000010000000C0000000400000A8000000001000000360000000500000AD0000000010000000300000009000000080000000300000001000000370000000100000010000000010000001800000009000000050000046C00000001000000380000000500000B500000000100000003000000090000000800000003000000010000000F000000010000000E0000000400000AE80000000500000AFC00000001000000030000000900000008000000030000000100000012000000010000000E0000000900000001000000390000000100000012000000010000001800000009000000050000046C0000000100000003000000090000000800000003000000010000000F000000010000000E0000000400000B680000000500000B7C0000000100000003000000090000000800000003000000010000000C0000000400000BB0000000010000003A0000000500000AD000000001000000030000000900000008000000030000000100000061000000060000006A000000010000000D000000010000006100000001000000620000000600000075000000010000006200000001000000250000000100000063000000060000007600000001000000630000000400000C34000000010000003B0000000500000AD000000001000000030000000900000008000000030000000100000062000000010000002200000001000000640000000600000077000000010000003C000000010000003D00000001000000640000000100000026000000060000006F0000000100000003000000090000000800000003000000010000000F00000001000000040000000100000065000000060000007000000001000000650000000400000CEC00000001000000040000000100000004000000010000006600000009000000010000003E000000060000006D0000000100000003000000090000000800000003000000010000000F00000001000000040000000100000067000000060000007000000001000000670000000400000D5C00000001000000040000000100000004000000010000006800000009000000010000003F000000060000006D0000000100000003000000090000000800000003000000010000000F00000001000000040000000100000069000000060000007000000001000000690000000400000DC000000001000000040000000100000040000000010000004100000006000000790000000100000003000000090000000800000003",
  "byteCodeLength": 3540,
  "symbols": {
    "__gintnl_SystemUInt32_16": {
      "name": "__gintnl_SystemUInt32_16",
      "type": "System.UInt32",
      "address": 56
    },
    "__intnl_SystemSingle_0": {
      "name": "__intnl_SystemSingle_0",
      "type": "System.Single",
      "address": 66
    },
    "__const_SystemString_0": {
      "name": "__const_SystemString_0",
      "type": "System.String",
      "address": 26
    },
    "__const_SystemSingle_0": {
      "name": "__const_SystemSingle_0",
      "type": "System.Single",
      "address": 19
    },
    "targetTime2": {
      "name": "targetTime2",
      "type": "System.Single",
      "address": 10
    },
    "targetTime1": {
      "name": "targetTime1",
      "type": "System.Single",
      "address": 9
    },
    "__intnl_SystemSingle_10": {
      "name": "__intnl_SystemSingle_10",
      "type": "System.Single",
      "address": 91
    },
    "__intnl_SystemSingle_11": {
      "name": "__intnl_SystemSingle_11",
      "type": "System.Single",
      "address": 93
    },
    "__intnl_SystemSingle_12": {
      "name": "__intnl_SystemSingle_12",
      "type": "System.Single",
      "address": 95
    },
    "__intnl_SystemSingle_13": {
      "name": "__intnl_SystemSingle_13",
      "type": "System.Single",
      "address": 96
    },
    "__intnl_SystemSingle_14": {
      "name": "__intnl_SystemSingle_14",
      "type": "System.Single",
      "address": 100
    },
    "__intnl_SystemSingle_8": {
      "name": "__intnl_SystemSingle_8",
      "type": "System.Single",
      "address": 89
    },
    "__gintnl_SystemUInt32_9": {
      "name": "__gintnl_SystemUInt32_9",
      "type": "System.UInt32",
      "address": 43
    },
    "__gintnl_SystemUInt32_8": {
      "name": "__gintnl_SystemUInt32_8",
      "type": "System.UInt32",
      "address": 42
    },
    "__gintnl_SystemUInt32_5": {
      "name": "__gintnl_SystemUInt32_5",
      "type": "System.UInt32",
      "address": 31
    },
    "__gintnl_SystemUInt32_4": {
      "name": "__gintnl_SystemUInt32_4",
      "type": "System.UInt32",
      "address": 30
    },
    "__gintnl_SystemUInt32_7": {
      "name": "__gintnl_SystemUInt32_7",
      "type": "System.UInt32",
      "address": 39
    },
    "__gintnl_SystemUInt32_6": {
      "name": "__gintnl_SystemUInt32_6",
      "type": "System.UInt32",
      "address": 32
    },
    "__gintnl_SystemUInt32_1": {
      "name": "__gintnl_SystemUInt32_1",
      "type": "System.UInt32",
      "address": 20
    },
    "__gintnl_SystemUInt32_0": {
      "name": "__gintnl_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 17
    },
    "__gintnl_SystemUInt32_3": {
      "name": "__gintnl_SystemUInt32_3",
      "type": "System.UInt32",
      "address": 23
    },
    "__gintnl_SystemUInt32_2": {
      "name": "__gintnl_SystemUInt32_2",
      "type": "System.UInt32",
      "address": 21
    },
    "__const_SystemBoolean_0": {
      "name": "__const_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 16
    },
    "__const_SystemBoolean_1": {
      "name": "__const_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 18
    },
    "__const_SystemString_5": {
      "name": "__const_SystemString_5",
      "type": "System.String",
      "address": 62
    },
    "__gintnl_SystemUInt32_13": {
      "name": "__gintnl_SystemUInt32_13",
      "type": "System.UInt32",
      "address": 52
    },
    "inputUseArgs": {
      "name": "inputUseArgs",
      "type": "VRC.Udon.Common.UdonInputEventArgs",
      "address": 29
    },
    "__const_VRCUdonCommonInterfacesNetworkEventTarget_0": {
      "name": "__const_VRCUdonCommonInterfacesNetworkEventTarget_0",
      "type": "VRC.Udon.Common.Interfaces.NetworkEventTarget",
      "address": 64
    },
    "__lcl_leaveTime_TextImage_SystemSingle_0": {
      "name": "__lcl_leaveTime_TextImage_SystemSingle_0",
      "type": "System.Single",
      "address": 98
    },
    "__intnl_SystemSingle_7": {
      "name": "__intnl_SystemSingle_7",
      "type": "System.Single",
      "address": 88
    },
    "__refl_typeid": {
      "name": "__refl_typeid",
      "type": "System.Int64",
      "address": 0
    },
    "__intnl_VRCUdonUdonBehaviour_2": {
      "name": "__intnl_VRCUdonUdonBehaviour_2",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 104
    },
    "__refl_typeids": {
      "name": "__refl_typeids",
      "type": "System.Int64[]",
      "address": 2
    },
    "__const_SystemUInt32_0": {
      "name": "__const_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 15
    },
    "__intnl_SystemBoolean_11": {
      "name": "__intnl_SystemBoolean_11",
      "type": "System.Boolean",
      "address": 103
    },
    "__gintnl_SystemUInt32_14": {
      "name": "__gintnl_SystemUInt32_14",
      "type": "System.UInt32",
      "address": 54
    },
    "__intnl_SystemSingle_2": {
      "name": "__intnl_SystemSingle_2",
      "type": "System.Single",
      "address": 70
    },
    "allTextImage": {
      "name": "allTextImage",
      "type": "UnityEngine.UI.Image",
      "address": 7
    },
    "__intnl_VRCUdonUdonBehaviour_1": {
      "name": "__intnl_VRCUdonUdonBehaviour_1",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 102
    },
    "__const_SystemString_2": {
      "name": "__const_SystemString_2",
      "type": "System.String",
      "address": 36
    },
    "__const_SystemSingle_2": {
      "name": "__const_SystemSingle_2",
      "type": "System.Single",
      "address": 34
    },
    "__intnl_UnityEngineGameObject_2": {
      "name": "__intnl_UnityEngineGameObject_2",
      "type": "UnityEngine.GameObject",
      "address": 79
    },
    "__intnl_UnityEngineGameObject_1": {
      "name": "__intnl_UnityEngineGameObject_1",
      "type": "UnityEngine.GameObject",
      "address": 77
    },
    "__intnl_UnityEngineGameObject_0": {
      "name": "__intnl_UnityEngineGameObject_0",
      "type": "UnityEngine.GameObject",
      "address": 75
    },
    "__intnl_SystemSingle_1": {
      "name": "__intnl_SystemSingle_1",
      "type": "System.Single",
      "address": 69
    },
    "__const_SystemString_7": {
      "name": "__const_SystemString_7",
      "type": "System.String",
      "address": 65
    },
    "isPickedUp": {
      "name": "isPickedUp",
      "type": "System.Boolean",
      "address": 12
    },
    "__const_SystemSingle_1": {
      "name": "__const_SystemSingle_1",
      "type": "System.Single",
      "address": 33
    },
    "__gintnl_SystemUInt32_11": {
      "name": "__gintnl_SystemUInt32_11",
      "type": "System.UInt32",
      "address": 50
    },
    "allIndicator": {
      "name": "allIndicator",
      "type": "UnityEngine.UI.Image",
      "address": 8
    },
    "__const_VRCUdonCommonEnumsEventTiming_0": {
      "name": "__const_VRCUdonCommonEnumsEventTiming_0",
      "type": "VRC.Udon.Common.Enums.EventTiming",
      "address": 38
    },
    "isIn_LoopTextImageActive": {
      "name": "isIn_LoopTextImageActive",
      "type": "System.Boolean",
      "address": 14
    },
    "__intnl_SystemSingle_9": {
      "name": "__intnl_SystemSingle_9",
      "type": "System.Single",
      "address": 90
    },
    "__intnl_returnJump_SystemUInt32_0": {
      "name": "__intnl_returnJump_SystemUInt32_0",
      "type": "System.UInt32",
      "address": 3
    },
    "__const_SystemString_4": {
      "name": "__const_SystemString_4",
      "type": "System.String",
      "address": 61
    },
    "__gintnl_SystemUInt32_19": {
      "name": "__gintnl_SystemUInt32_19",
      "type": "System.UInt32",
      "address": 59
    },
    "__const_SystemSingle_4": {
      "name": "__const_SystemSingle_4",
      "type": "System.Single",
      "address": 44
    },
    "__gintnl_SystemUInt32_12": {
      "name": "__gintnl_SystemUInt32_12",
      "type": "System.UInt32",
      "address": 51
    },
    "__intnl_SystemSingle_4": {
      "name": "__intnl_SystemSingle_4",
      "type": "System.Single",
      "address": 72
    },
    "ownIndicator": {
      "name": "ownIndicator",
      "type": "UnityEngine.UI.Image",
      "address": 6
    },
    "__intnl_SystemBoolean_12": {
      "name": "__intnl_SystemBoolean_12",
      "type": "System.Boolean",
      "address": 105
    },
    "__gintnl_SystemUInt32_17": {
      "name": "__gintnl_SystemUInt32_17",
      "type": "System.UInt32",
      "address": 57
    },
    "__refl_typename": {
      "name": "__refl_typename",
      "type": "System.String",
      "address": 1
    },
    "__1_isActive__param": {
      "name": "__1_isActive__param",
      "type": "System.Boolean",
      "address": 24
    },
    "__0_ownIndicatorValue__param": {
      "name": "__0_ownIndicatorValue__param",
      "type": "System.Single",
      "address": 40
    },
    "__intnl_SystemSingle_3": {
      "name": "__intnl_SystemSingle_3",
      "type": "System.Single",
      "address": 71
    },
    "__const_SystemString_1": {
      "name": "__const_SystemString_1",
      "type": "System.String",
      "address": 27
    },
    "__const_SystemSingle_3": {
      "name": "__const_SystemSingle_3",
      "type": "System.Single",
      "address": 37
    },
    "__0_isActive__param": {
      "name": "__0_isActive__param",
      "type": "System.Boolean",
      "address": 22
    },
    "targetTime_TextImage": {
      "name": "targetTime_TextImage",
      "type": "System.Single",
      "address": 13
    },
    "__const_SystemString_6": {
      "name": "__const_SystemString_6",
      "type": "System.String",
      "address": 63
    },
    "__gintnl_SystemUInt32_10": {
      "name": "__gintnl_SystemUInt32_10",
      "type": "System.UInt32",
      "address": 48
    },
    "__lcl_leaveTime2_SystemSingle_0": {
      "name": "__lcl_leaveTime2_SystemSingle_0",
      "type": "System.Single",
      "address": 86
    },
    "__lcl_leaveTime2_SystemSingle_1": {
      "name": "__lcl_leaveTime2_SystemSingle_1",
      "type": "System.Single",
      "address": 92
    },
    "inputUseBoolValue": {
      "name": "inputUseBoolValue",
      "type": "System.Boolean",
      "address": 28
    },
    "__0_allIndicatorValue__param": {
      "name": "__0_allIndicatorValue__param",
      "type": "System.Single",
      "address": 41
    },
    "isInInteract": {
      "name": "isInInteract",
      "type": "System.Boolean",
      "address": 11
    },
    "ownTextImage": {
      "name": "ownTextImage",
      "type": "UnityEngine.UI.Image",
      "address": 5
    },
    "__intnl_SystemSingle_6": {
      "name": "__intnl_SystemSingle_6",
      "type": "System.Single",
      "address": 83
    },
    "penManager": {
      "name": "penManager",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 4
    },
    "__lcl_leaveTime1_SystemSingle_0": {
      "name": "__lcl_leaveTime1_SystemSingle_0",
      "type": "System.Single",
      "address": 85
    },
    "__gintnl_SystemUInt32_18": {
      "name": "__gintnl_SystemUInt32_18",
      "type": "System.UInt32",
      "address": 58
    },
    "__intnl_SystemBoolean_10": {
      "name": "__intnl_SystemBoolean_10",
      "type": "System.Boolean",
      "address": 101
    },
    "__gintnl_SystemUInt32_15": {
      "name": "__gintnl_SystemUInt32_15",
      "type": "System.UInt32",
      "address": 55
    },
    "__const_SystemInt32_0": {
      "name": "__const_SystemInt32_0",
      "type": "System.Int32",
      "address": 47
    },
    "__intnl_SystemBoolean_8": {
      "name": "__intnl_SystemBoolean_8",
      "type": "System.Boolean",
      "address": 94
    },
    "__intnl_SystemBoolean_9": {
      "name": "__intnl_SystemBoolean_9",
      "type": "System.Boolean",
      "address": 99
    },
    "__intnl_SystemBoolean_0": {
      "name": "__intnl_SystemBoolean_0",
      "type": "System.Boolean",
      "address": 68
    },
    "__intnl_SystemBoolean_1": {
      "name": "__intnl_SystemBoolean_1",
      "type": "System.Boolean",
      "address": 73
    },
    "__intnl_SystemBoolean_2": {
      "name": "__intnl_SystemBoolean_2",
      "type": "System.Boolean",
      "address": 74
    },
    "__intnl_SystemBoolean_3": {
      "name": "__intnl_SystemBoolean_3",
      "type": "System.Boolean",
      "address": 76
    },
    "__intnl_SystemBoolean_4": {
      "name": "__intnl_SystemBoolean_4",
      "type": "System.Boolean",
      "address": 78
    },
    "__intnl_SystemBoolean_5": {
      "name": "__intnl_SystemBoolean_5",
      "type": "System.Boolean",
      "address": 80
    },
    "__intnl_SystemBoolean_6": {
      "name": "__intnl_SystemBoolean_6",
      "type": "System.Boolean",
      "address": 82
    },
    "__intnl_SystemBoolean_7": {
      "name": "__intnl_SystemBoolean_7",
      "type": "System.Boolean",
      "address": 87
    },
    "__lcl_time_SystemSingle_1": {
      "name": "__lcl_time_SystemSingle_1",
      "type": "System.Single",
      "address": 97
    },
    "__lcl_time_SystemSingle_0": {
      "name": "__lcl_time_SystemSingle_0",
      "type": "System.Single",
      "address": 84
    },
    "__intnl_SystemSingle_5": {
      "name": "__intnl_SystemSingle_5",
      "type": "System.Single",
      "address": 81
    },
    "__intnl_VRCUdonUdonBehaviour_0": {
      "name": "__intnl_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 67
    },
    "__const_SystemString_3": {
      "name": "__const_SystemString_3",
      "type": "System.String",
      "address": 46
    },
    "__this_VRCUdonUdonBehaviour_3": {
      "name": "__this_VRCUdonUdonBehaviour_3",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 49
    },
    "__this_VRCUdonUdonBehaviour_2": {
      "name": "__this_VRCUdonUdonBehaviour_2",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 45
    },
    "__this_VRCUdonUdonBehaviour_1": {
      "name": "__this_VRCUdonUdonBehaviour_1",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 35
    },
    "__this_VRCUdonUdonBehaviour_0": {
      "name": "__this_VRCUdonUdonBehaviour_0",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 25
    },
    "__this_VRCUdonUdonBehaviour_5": {
      "name": "__this_VRCUdonUdonBehaviour_5",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 60
    },
    "__this_VRCUdonUdonBehaviour_4": {
      "name": "__this_VRCUdonUdonBehaviour_4",
      "type": "VRC.Udon.UdonBehaviour",
      "address": 53
    }
  },
  "entryPoints": [
    {
      "name": "OnPenPickup",
      "address": 0
    },
    {
      "name": "OnPenDrop",
      "address": 64
    },
    {
      "name": "_start",
      "address": 176
    },
    {
      "name": "_inputUse",
      "address": 352
    },
    {
      "name": "_interact",
      "address": 588
    },
    {
      "name": "_LoopIndicator1",
      "address": 1532
    },
    {
      "name": "_LoopIndicator2",
      "address": 2152
    },
    {
      "name": "_LoopTextImageActive",
      "address": 2888
    }
  ],
  "heapInitialValues": {
    "0": {
      "address": 0,
      "type": "System.Int64",
      "value": {
        "isSerializable": true,
        "value": 3142330649835999805
      }
    },
    "1": {
      "address": 1,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "QvPen.Udon.UI.QvPen_ClearButton"
      }
    },
    "2": {
      "address": 2,
      "type": "System.Int64[]",
      "value": {
        "isSerializable": true,
        "value": [
          3142330649835999805,
          -5351660574498133152
        ]
      }
    },
    "3": {
      "address": 3,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "4": {
      "address": 4,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "5": {
      "address": 5,
      "type": "UnityEngine.UI.Image",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "6": {
      "address": 6,
      "type": "UnityEngine.UI.Image",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "7": {
      "address": 7,
      "type": "UnityEngine.UI.Image",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "8": {
      "address": 8,
      "type": "UnityEngine.UI.Image",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "9": {
      "address": 9,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "10": {
      "address": 10,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "11": {
      "address": 11,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "12": {
      "address": 12,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "13": {
      "address": 13,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "14": {
      "address": 14,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "15": {
      "address": 15,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 4294967295
      }
    },
    "16": {
      "address": 16,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": true
      }
    },
    "17": {
      "address": 17,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 44
      }
    },
    "18": {
      "address": 18,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "19": {
      "address": 19,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 5.0
      }
    },
    "20": {
      "address": 20,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 156
      }
    },
    "21": {
      "address": 21,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 220
      }
    },
    "22": {
      "address": 22,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "23": {
      "address": 23,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 256
      }
    },
    "24": {
      "address": 24,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "25": {
      "address": 25,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "26": {
      "address": 26,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_listener__param"
      }
    },
    "27": {
      "address": 27,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "__0_Register"
      }
    },
    "28": {
      "address": 28,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "29": {
      "address": 29,
      "type": "VRC.Udon.Common.UdonInputEventArgs",
      "value": {
        "isSerializable": true,
        "value": {
          "eventType": 0,
          "boolValue": false,
          "floatValue": 0.0,
          "handType": 0
        }
      }
    },
    "30": {
      "address": 30,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 512
      }
    },
    "31": {
      "address": 31,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 568
      }
    },
    "32": {
      "address": 32,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 652
      }
    },
    "33": {
      "address": 33,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.31
      }
    },
    "34": {
      "address": 34,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 2.0
      }
    },
    "35": {
      "address": 35,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "36": {
      "address": 36,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_LoopIndicator1"
      }
    },
    "37": {
      "address": 37,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "38": {
      "address": 38,
      "type": "VRC.Udon.Common.Enums.EventTiming",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "39": {
      "address": 39,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 852
      }
    },
    "40": {
      "address": 40,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "41": {
      "address": 41,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "42": {
      "address": 42,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 1728
      }
    },
    "43": {
      "address": 43,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 1848
      }
    },
    "44": {
      "address": 44,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 1.0
      }
    },
    "45": {
      "address": 45,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "46": {
      "address": 46,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_LoopIndicator2"
      }
    },
    "47": {
      "address": 47,
      "type": "System.Int32",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "48": {
      "address": 48,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2092
      }
    },
    "49": {
      "address": 49,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "50": {
      "address": 50,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2316
      }
    },
    "51": {
      "address": 51,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2372
      }
    },
    "52": {
      "address": 52,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2512
      }
    },
    "53": {
      "address": 53,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "54": {
      "address": 54,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2668
      }
    },
    "55": {
      "address": 55,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2724
      }
    },
    "56": {
      "address": 56,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2740
      }
    },
    "57": {
      "address": 57,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2868
      }
    },
    "58": {
      "address": 58,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 2972
      }
    },
    "59": {
      "address": 59,
      "type": "System.UInt32",
      "value": {
        "isSerializable": true,
        "value": 3104
      }
    },
    "60": {
      "address": 60,
      "type": "VRC.Udon.Common.UdonGameObjectComponentHeapReference",
      "value": {
        "isSerializable": true,
        "value": {
          "type": "VRC.Udon.UdonBehaviour, VRC.Udon, Version=0.0.0.0, Culture=neutral, PublicKeyToken=null"
        }
      }
    },
    "61": {
      "address": 61,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "_LoopTextImageActive"
      }
    },
    "62": {
      "address": 62,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "EraseOwnInk"
      }
    },
    "63": {
      "address": 63,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UndoDraw"
      }
    },
    "64": {
      "address": 64,
      "type": "VRC.Udon.Common.Interfaces.NetworkEventTarget",
      "value": {
        "isSerializable": true,
        "value": 0
      }
    },
    "65": {
      "address": 65,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "Clear"
      }
    },
    "66": {
      "address": 66,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "67": {
      "address": 67,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "68": {
      "address": 68,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "69": {
      "address": 69,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "70": {
      "address": 70,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "71": {
      "address": 71,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "72": {
      "address": 72,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "73": {
      "address": 73,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "74": {
      "address": 74,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "75": {
      "address": 75,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "76": {
      "address": 76,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "77": {
      "address": 77,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "78": {
      "address": 78,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "79": {
      "address": 79,
      "type": "UnityEngine.GameObject",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "80": {
      "address": 80,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "81": {
      "address": 81,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "82": {
      "address": 82,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "83": {
      "address": 83,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "84": {
      "address": 84,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "85": {
      "address": 85,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "86": {
      "address": 86,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "87": {
      "address": 87,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "88": {
      "address": 88,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "89": {
      "address": 89,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "90": {
      "address": 90,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "91": {
      "address": 91,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "92": {
      "address": 92,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "93": {
      "address": 93,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "94": {
      "address": 94,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "95": {
      "address": 95,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "96": {
      "address": 96,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "97": {
      "address": 97,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "98": {
      "address": 98,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "99": {
      "address": 99,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "100": {
      "address": 100,
      "type": "System.Single",
      "value": {
        "isSerializable": true,
        "value": 0.0
      }
    },
    "101": {
      "address": 101,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "102": {
      "address": 102,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "103": {
      "address": 103,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "104": {
      "address": 104,
      "type": "VRC.Udon.UdonBehaviour",
      "value": {
        "isSerializable": true,
        "value": null
      }
    },
    "105": {
      "address": 105,
      "type": "System.Boolean",
      "value": {
        "isSerializable": true,
        "value": false
      }
    },
    "106": {
      "address": 106,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineTime.__get_time__SystemSingle"
      }
    },
    "107": {
      "address": 107,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemSingle.__op_Addition__SystemSingle_SystemSingle__SystemSingle"
      }
    },
    "108": {
      "address": 108,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SetProgramVariable__SystemString_SystemObject__SystemVoid"
      }
    },
    "109": {
      "address": 109,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomEvent__SystemString__SystemVoid"
      }
    },
    "110": {
      "address": 110,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemSingle.__op_LessThan__SystemSingle_SystemSingle__SystemBoolean"
      }
    },
    "111": {
      "address": 111,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomEventDelayedSeconds__SystemString_SystemSingle_VRCUdonCommonEnumsEventTiming__SystemVoid"
      }
    },
    "112": {
      "address": 112,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCSDKBaseUtilities.__IsValid__SystemObject__SystemBoolean"
      }
    },
    "113": {
      "address": 113,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineComponent.__get_gameObject__UnityEngineGameObject"
      }
    },
    "114": {
      "address": 114,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineGameObject.__SetActive__SystemBoolean__SystemVoid"
      }
    },
    "115": {
      "address": 115,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineUIImage.__set_fillAmount__SystemSingle__SystemVoid"
      }
    },
    "116": {
      "address": 116,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "UnityEngineMathf.__Clamp01__SystemSingle__SystemSingle"
      }
    },
    "117": {
      "address": 117,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemSingle.__op_Subtraction__SystemSingle_SystemSingle__SystemSingle"
      }
    },
    "118": {
      "address": 118,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemSingle.__op_LessThanOrEqual__SystemSingle_SystemSingle__SystemBoolean"
      }
    },
    "119": {
      "address": 119,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "SystemSingle.__op_Division__SystemSingle_SystemSingle__SystemSingle"
      }
    },
    "120": {
      "address": 120,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomEventDelayedFrames__SystemString_SystemInt32_VRCUdonCommonEnumsEventTiming__SystemVoid"
      }
    },
    "121": {
      "address": 121,
      "type": "System.String",
      "value": {
        "isSerializable": true,
        "value": "VRCUdonCommonInterfacesIUdonEventReceiver.__SendCustomNetworkEvent__VRCUdonCommonInterfacesNetworkEventTarget_SystemString__SystemVoid"
      }
    }
  }
}
```