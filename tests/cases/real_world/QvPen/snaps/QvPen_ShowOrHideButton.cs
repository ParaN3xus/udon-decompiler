// Decompiled Udon Program
// This is pseudo-code and may not compile directly

namespace QvPen.UdonScript.UI
{
    public class QvPen_ShowOrHideButton : UdonSharpBehaviour
    {
        UnityEngine.GameObject[] gameObjects = null /* [] */;
        System.Boolean isShown = true;
        UnityEngine.GameObject displayObjectOn = null;
        UnityEngine.GameObject displayObjectOff = null;

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
            System.Int32 __intnl_SystemInt32_0 = 0;
            System.Int32 __intnl_SystemInt32_1 = 0;
            UnityEngine.GameObject __lcl_go_UnityEngineGameObject_0 = null;

            if (displayObjectOn)
            {
                displayObjectOn.SetActive(isShown);
            }
            if (displayObjectOff)
            {
                displayObjectOff.SetActive(!isShown);
            }
            __intnl_SystemInt32_0 = gameObjects.Length;
            __intnl_SystemInt32_1 = 0;
            while (__intnl_SystemInt32_1 < __intnl_SystemInt32_0)
            {
                __lcl_go_UnityEngineGameObject_0 = gameObjects.Get(__intnl_SystemInt32_1);
                if (__lcl_go_UnityEngineGameObject_0)
                {
                    __lcl_go_UnityEngineGameObject_0.SetActive(isShown);
                }
                __intnl_SystemInt32_1 = __intnl_SystemInt32_1 + 1;
            }
            return;
        }
    }
}