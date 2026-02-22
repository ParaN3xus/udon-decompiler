// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class ComplexControlFlow : UdonSharpBehaviour
{
    System.Int32 limit = 5;
    System.Int32 seed = 3;

    public void _start()
    {
        System.Int32 __lcl_i_SystemInt32_0 = 0;
        System.Int32 __lcl_j_SystemInt32_0 = 0;
        System.Int32 __lcl_sum_SystemInt32_0 = 0;
        System.Boolean __lcl_flag_SystemBoolean_0 = false;
        System.Boolean __intnl_SystemBoolean_0 = false;
        System.Boolean __intnl_SystemBoolean_1 = false;
        System.Boolean __intnl_SystemBoolean_2 = false;
        System.Int32 __lcl_local_SystemInt32_0 = 0;
        System.Int32 __lcl_k_SystemInt32_0 = 0;

        __lcl_i_SystemInt32_0 = 0;
        __lcl_j_SystemInt32_0 = 0;
        __lcl_sum_SystemInt32_0 = 0;
        __lcl_flag_SystemBoolean_0 = false;
        while (true)
        {
            __lcl_local_SystemInt32_0 = __lcl_i_SystemInt32_0 * seed;
            if (__lcl_local_SystemInt32_0 % 2 == 0)
            {
                __intnl_SystemBoolean_0 = __lcl_i_SystemInt32_0 < limit;
            }
            if (__intnl_SystemBoolean_0)
            {
                __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 + __lcl_local_SystemInt32_0;
            }
            else
            {
                if (!(__lcl_local_SystemInt32_0 % 3 == 0))
                {
                    __intnl_SystemBoolean_1 = __lcl_i_SystemInt32_0 == 0;
                }
                if (__intnl_SystemBoolean_1)
                {
                    __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 - __lcl_local_SystemInt32_0;
                }
                else
                {
                    __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 + 1;
                }
            }
            __lcl_j_SystemInt32_0 = 0;
            while (__lcl_j_SystemInt32_0 < 4)
            {
                if (__lcl_j_SystemInt32_0 == 2)
                {
                    __lcl_j_SystemInt32_0 = __lcl_j_SystemInt32_0 + 1;
                }
                else
                {
                    __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 + __lcl_j_SystemInt32_0;
                    if (__lcl_sum_SystemInt32_0 > 50)
                    {
                        __lcl_flag_SystemBoolean_0 = true;
                        break;
                    }
                    __lcl_j_SystemInt32_0 = __lcl_j_SystemInt32_0 + 1;
                }
            }
            if (!__lcl_flag_SystemBoolean_0)
            {
                if (__lcl_i_SystemInt32_0 == 0)
                {
                    __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 + 10;
                }
                else
                {
                    if (!(__lcl_i_SystemInt32_0 == 1) & !(__lcl_i_SystemInt32_0 == 2))
                    {
                        __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 + 30;
                    }
                    else
                    {
                        __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 + 20;
                    }
                }
                __lcl_k_SystemInt32_0 = 0;
                __intnl_SystemBoolean_3 = __lcl_k_SystemInt32_0 < 3;
                while (__intnl_SystemBoolean_3)
                {
                    __intnl_SystemBoolean_4 = __lcl_k_SystemInt32_0 == 1;
                    if (!__intnl_SystemBoolean_4)
                    {
                        __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 + __lcl_k_SystemInt32_0;
                    }
                    else
                    {
                    }
                    __lcl_k_SystemInt32_0 = __lcl_k_SystemInt32_0 + 1;
                }
                __lcl_i_SystemInt32_0 = __lcl_i_SystemInt32_0 + 1;
                __intnl_SystemBoolean_0 = __lcl_i_SystemInt32_0 < limit;
                if (!__intnl_SystemBoolean_0)
                {
                    break;
                }
                else
                {
                    continue;
                }
            }
            else
            {
                break;
            }
        }
        if (__lcl_sum_SystemInt32_0 > 0)
        {
            __intnl_SystemBoolean_2 = true;
        }
        else
        {
            __intnl_SystemBoolean_2 = false;
        }
        if (__intnl_SystemBoolean_2)
        {
            UnityEngine.Debug.Log("Positive sum");
        }
        else
        {
            UnityEngine.Debug.Log("Non-positive sum");
        }
        if (__lcl_sum_SystemInt32_0 > 100)
        {
            UnityEngine.Debug.Log("Large sum");
        }
        else
        {
            UnityEngine.Debug.Log(__lcl_sum_SystemInt32_0);
        }
        return;
    }
}