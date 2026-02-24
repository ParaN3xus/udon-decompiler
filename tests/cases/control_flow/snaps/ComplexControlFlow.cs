// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class ComplexControlFlow : UdonSharpBehaviour
{
    System.Int32 limit = 5;
    System.Int32 seed = 3;

    public void _start()
    {
        System.Int32 __lcl_i_SystemInt32_0;
        System.Int32 __lcl_j_SystemInt32_0;
        System.Int32 __lcl_sum_SystemInt32_0;
        System.Boolean __lcl_flag_SystemBoolean_0;
        System.Boolean __intnl_SystemBoolean_0;
        System.Boolean __intnl_SystemBoolean_1;
        System.Boolean __intnl_SystemBoolean_2;
        System.Boolean __intnl_SystemBoolean_3;
        System.Int32 __lcl_local_SystemInt32_0;
        System.Boolean __intnl_SystemBoolean_4;
        System.Int32 __lcl_k_SystemInt32_0;

        __lcl_i_SystemInt32_0 = 0;
        __lcl_j_SystemInt32_0 = 0;
        __lcl_sum_SystemInt32_0 = 0;
        __lcl_flag_SystemBoolean_0 = false;
        while (true)
        {
            __lcl_local_SystemInt32_0 = __lcl_i_SystemInt32_0 * seed;
            __intnl_SystemBoolean_0 = __lcl_local_SystemInt32_0 % 2 == 0;
            if (__intnl_SystemBoolean_0)
            {
                __intnl_SystemBoolean_0 = __lcl_i_SystemInt32_0 < limit;
            }
            if (!__intnl_SystemBoolean_0)
            {
                __intnl_SystemBoolean_1 = __lcl_local_SystemInt32_0 % 3 == 0;
                if (!__intnl_SystemBoolean_1)
                {
                    __intnl_SystemBoolean_1 = __lcl_i_SystemInt32_0 == 0;
                }
                else
                {
                }
                if (!__intnl_SystemBoolean_1)
                {
                    __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 + 1;
                }
                else
                {
                    __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 - __lcl_local_SystemInt32_0;
                }
            }
            else
            {
                __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 + __lcl_local_SystemInt32_0;
            }
            __lcl_j_SystemInt32_0 = 0;
            __intnl_SystemBoolean_2 = __lcl_j_SystemInt32_0 < 4;
            while (__intnl_SystemBoolean_2)
            {
                __intnl_SystemBoolean_3 = __lcl_j_SystemInt32_0 == 2;
                if (!__intnl_SystemBoolean_3)
                {
                    __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 + __lcl_j_SystemInt32_0;
                    __intnl_SystemBoolean_4 = __lcl_sum_SystemInt32_0 > 50;
                    if (!__intnl_SystemBoolean_4)
                    {
                        __lcl_j_SystemInt32_0 = __lcl_j_SystemInt32_0 + 1;
                    }
                    else
                    {
                        __lcl_flag_SystemBoolean_0 = true;
                        goto label_bb_00000013;
                    }
                }
                else
                {
                    __lcl_j_SystemInt32_0 = __lcl_j_SystemInt32_0 + 1;
                }
            }
        label_bb_00000013:
            if (!__lcl_flag_SystemBoolean_0)
            {
                __intnl_SystemBoolean_3 = __lcl_i_SystemInt32_0 == 0;
                if (!__intnl_SystemBoolean_3)
                {
                    __intnl_SystemBoolean_4 = __lcl_i_SystemInt32_0 == 1;
                    if (!__intnl_SystemBoolean_4)
                    {
                        if (!(__lcl_i_SystemInt32_0 == 2))
                        {
                            __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 + 30;
                            goto label_bb_0000001e;
                        }
                        else
                        {
                        }
                    }
                    else
                    {
                    }
                    __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 + 20;
                }
                else
                {
                    __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 + 10;
                }
            label_bb_0000001e:
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
        __intnl_SystemBoolean_1 = __lcl_sum_SystemInt32_0 > 0;
        if (!__intnl_SystemBoolean_1)
        {
            __intnl_SystemBoolean_2 = false;
        }
        else
        {
            __intnl_SystemBoolean_2 = true;
        }
        if (!__intnl_SystemBoolean_2)
        {
            UnityEngine.Debug.Log("Non-positive sum");
        }
        else
        {
            UnityEngine.Debug.Log("Positive sum");
        }
        __intnl_SystemBoolean_3 = __lcl_sum_SystemInt32_0 > 100;
        if (!__intnl_SystemBoolean_3)
        {
            UnityEngine.Debug.Log(__lcl_sum_SystemInt32_0);
            return;
        }
        else
        {
            UnityEngine.Debug.Log("Large sum");
            return;
        }
    }
}