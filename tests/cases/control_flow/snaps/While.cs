// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class While : UdonSharpBehaviour
{
    public void _start()
    {
        System.Int32 __lcl_counter_SystemInt32_0;
        System.Int32 __lcl_sum_SystemInt32_0;
        __lcl_counter_SystemInt32_0 = 0;
        __lcl_sum_SystemInt32_0 = 0;
        while (__lcl_counter_SystemInt32_0 < 10)
        {
            __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 + __lcl_counter_SystemInt32_0;
            __lcl_counter_SystemInt32_0 = __lcl_counter_SystemInt32_0 + 1;
            if (__lcl_sum_SystemInt32_0 > 20)
            {
                UnityEngine.Debug.Log("Sum is growing");
            }
        }
        UnityEngine.Debug.Log(__lcl_sum_SystemInt32_0);
        return;
    }
}