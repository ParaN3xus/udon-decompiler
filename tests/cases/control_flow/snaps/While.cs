// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class While : UdonSharpBehaviour
{
    public void _start()
    {
        System.Int32 __lcl_counter_SystemInt32_0;
        System.Int32 __lcl_sum_SystemInt32_0;
        System.Int32 __scfg_backedge_var_0__;
        System.Boolean __scfg_loop_cont_1__;
        __lcl_counter_SystemInt32_0 = 0;
        __lcl_sum_SystemInt32_0 = 0;
        __scfg_loop_cont_1__ = true;
        while (__scfg_loop_cont_1__)
        {
            if (__lcl_counter_SystemInt32_0 < 10)
            {
                __lcl_sum_SystemInt32_0 = __lcl_sum_SystemInt32_0 + __lcl_counter_SystemInt32_0;
                __lcl_counter_SystemInt32_0 = __lcl_counter_SystemInt32_0 + 1;
                if (__lcl_sum_SystemInt32_0 > 20)
                {
                    UnityEngine.Debug.Log("Sum is growing");
                }
                __scfg_backedge_var_0__ = 0;
            }
            else
            {
                __scfg_backedge_var_0__ = 1;
            }
            __scfg_loop_cont_1__ = __scfg_backedge_var_0__ == 0;
        }
        UnityEngine.Debug.Log(__lcl_sum_SystemInt32_0);
        return;
    }
}