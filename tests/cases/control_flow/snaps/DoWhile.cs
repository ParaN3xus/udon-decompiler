// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class DoWhile : UdonSharpBehaviour
{
    public void _start()
    {
        System.Int32 __lcl_i_SystemInt32_0;
        __lcl_i_SystemInt32_0 = 0;
        do
        {
            UnityEngine.Debug.Log(__lcl_i_SystemInt32_0);
            __lcl_i_SystemInt32_0 = __lcl_i_SystemInt32_0 + 2;
        }
        while (__lcl_i_SystemInt32_0 < 10);
        UnityEngine.Debug.Log("Loop finished");
        return;
    }
}