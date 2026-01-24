// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class If : UdonSharpBehaviour
{
    public void _start()
    {
        System.Int32 __lcl_a_SystemInt32_0;
        __lcl_a_SystemInt32_0 = 10;
        if (__lcl_a_SystemInt32_0 > 5)
        {
            UnityEngine.Debug.Log("Condition is true");
            __lcl_a_SystemInt32_0 = __lcl_a_SystemInt32_0 * 2;
        }
        UnityEngine.Debug.Log("Finished");
        return;
    }
}