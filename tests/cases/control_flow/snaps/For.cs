// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class For : UdonSharpBehaviour
{
    public void _start()
    {
        System.Int32[] __lcl_numbers_SystemInt32Array_0;
        System.Int32 __lcl_total_SystemInt32_0;
        System.Int32 __lcl_i_SystemInt32_0;
        __lcl_numbers_SystemInt32Array_0 = new System.Int32[](5);
        __lcl_numbers_SystemInt32Array_0.Set(0, 1);
        __lcl_numbers_SystemInt32Array_0.Set(1, 2);
        __lcl_numbers_SystemInt32Array_0.Set(2, 3);
        __lcl_numbers_SystemInt32Array_0.Set(3, 4);
        __lcl_numbers_SystemInt32Array_0.Set(4, 5);
        __lcl_total_SystemInt32_0 = 0;
        __lcl_i_SystemInt32_0 = 0;
        while (__lcl_i_SystemInt32_0 < __lcl_numbers_SystemInt32Array_0.Length)
        {
            __lcl_total_SystemInt32_0 = __lcl_total_SystemInt32_0 + __lcl_numbers_SystemInt32Array_0.Get(__lcl_i_SystemInt32_0);
            UnityEngine.Debug.Log(__lcl_i_SystemInt32_0);
            __lcl_i_SystemInt32_0 = __lcl_i_SystemInt32_0 + 1;
        }
        UnityEngine.Debug.Log(__lcl_total_SystemInt32_0);
        return;
    }
}