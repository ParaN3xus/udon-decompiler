// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.UdonScript.UI
{
    public class QvPen_ClearAllButton : UdonSharpBehaviour
    {
        VRC.Udon.UdonBehaviour settings = null;

        public void _interact()
        {
            VRC.Udon.UdonBehaviour __lcl_penManager_VRCUdonUdonBehaviour_0;
            System.Int32 __intnl_SystemInt32_1;
            __intnl_SystemInt32_1 = 0;
            while (__intnl_SystemInt32_1 < settings.GetProgramVariable("penManagers").Length)
            {
                __lcl_penManager_VRCUdonUdonBehaviour_0 = settings.GetProgramVariable("penManagers").Get(__intnl_SystemInt32_1);
                if ((UnityEngine.Object)__lcl_penManager_VRCUdonUdonBehaviour_0)
                {
                    __lcl_penManager_VRCUdonUdonBehaviour_0.SendCustomEvent("Clear");
                }
                __intnl_SystemInt32_1 = __intnl_SystemInt32_1 + 1;
            }
            return;
        }
    }
}