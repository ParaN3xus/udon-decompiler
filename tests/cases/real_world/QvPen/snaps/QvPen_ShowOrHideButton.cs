// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.UdonScript.UI
{
    public class QvPen_ShowOrHideButton : UdonSharpBehaviour
    {
        UnityEngine.GameObject displayObjectOff = null;
        UnityEngine.GameObject[] gameObjects = null /* [] */;
        UnityEngine.GameObject displayObjectOn = null;
        System.Boolean isShown = true;

        public void _start()
        {
            function_0();
            return;
        }

        public void _interact()
        {
            isShown = isShown ^ true;
            function_0();
            return;
        }

        void function_0()
        {
            System.Int32 __intnl_SystemInt32_1;
            UnityEngine.GameObject __lcl_go_UnityEngineGameObject_0;
            if ((UnityEngine.Object)displayObjectOn)
            {
                displayObjectOn.SetActive(isShown);
            }
            if ((UnityEngine.Object)displayObjectOff)
            {
                displayObjectOff.SetActive(!isShown);
            }
            __intnl_SystemInt32_1 = 0;
            while (__intnl_SystemInt32_1 < gameObjects.Length)
            {
                __lcl_go_UnityEngineGameObject_0 = gameObjects.Get(__intnl_SystemInt32_1);
                if ((UnityEngine.Object)__lcl_go_UnityEngineGameObject_0)
                {
                    __lcl_go_UnityEngineGameObject_0.SetActive(isShown);
                }
                __intnl_SystemInt32_1 = __intnl_SystemInt32_1 + 1;
            }
            return;
        }
    }
}