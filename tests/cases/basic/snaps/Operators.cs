// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class Operators : UdonSharpBehaviour
{
    public void _start()
    {
        System.Int32 __lcl_a_SystemInt32_0;
        System.Int32 __lcl_b_SystemInt32_0;
        System.Int32 __lcl_add_SystemInt32_0;
        System.Int32 __lcl_sub_SystemInt32_0;
        System.Int32 __lcl_mul_SystemInt32_0;
        System.Int32 __lcl_div_SystemInt32_0;
        System.Int32 __lcl_mod_SystemInt32_0;
        System.Int32 __lcl_neg_SystemInt32_0;
        System.Boolean __lcl_flag_SystemBoolean_0;
        System.Boolean __lcl_notFlag_SystemBoolean_0;
        System.Int32 __lcl_bitNot_SystemInt32_0;
        System.Int32 __lcl_bitAnd_SystemInt32_0;
        System.Int32 __lcl_bitOr_SystemInt32_0;
        System.Int32 __lcl_bitXor_SystemInt32_0;
        System.Int32 __lcl_lShift_SystemInt32_0;
        System.Int32 __lcl_rShift_SystemInt32_0;
        System.Boolean __lcl_eq_SystemBoolean_0;
        System.Boolean __lcl_neq_SystemBoolean_0;
        System.Boolean __lcl_gt_SystemBoolean_0;
        System.Boolean __lcl_gte_SystemBoolean_0;
        System.Boolean __lcl_lt_SystemBoolean_0;
        System.Boolean __lcl_lte_SystemBoolean_0;
        __lcl_a_SystemInt32_0 = 10;
        __lcl_b_SystemInt32_0 = 3;
        __lcl_add_SystemInt32_0 = __lcl_a_SystemInt32_0 + __lcl_b_SystemInt32_0;
        __lcl_sub_SystemInt32_0 = __lcl_a_SystemInt32_0 - __lcl_b_SystemInt32_0;
        __lcl_mul_SystemInt32_0 = __lcl_a_SystemInt32_0 * __lcl_b_SystemInt32_0;
        __lcl_div_SystemInt32_0 = __lcl_a_SystemInt32_0 / __lcl_b_SystemInt32_0;
        __lcl_mod_SystemInt32_0 = __lcl_a_SystemInt32_0 % __lcl_b_SystemInt32_0;
        __lcl_neg_SystemInt32_0 = -__lcl_a_SystemInt32_0;
        __lcl_flag_SystemBoolean_0 = true;
        __lcl_notFlag_SystemBoolean_0 = !__lcl_flag_SystemBoolean_0;
        __lcl_bitNot_SystemInt32_0 = __lcl_a_SystemInt32_0 ^ -1;
        __lcl_bitAnd_SystemInt32_0 = __lcl_a_SystemInt32_0 & __lcl_b_SystemInt32_0;
        __lcl_bitOr_SystemInt32_0 = __lcl_a_SystemInt32_0 | __lcl_b_SystemInt32_0;
        __lcl_bitXor_SystemInt32_0 = __lcl_a_SystemInt32_0 ^ __lcl_b_SystemInt32_0;
        __lcl_lShift_SystemInt32_0 = __lcl_a_SystemInt32_0 << 1;
        __lcl_rShift_SystemInt32_0 = __lcl_a_SystemInt32_0 >> 1;
        __lcl_eq_SystemBoolean_0 = __lcl_a_SystemInt32_0 == __lcl_b_SystemInt32_0;
        __lcl_neq_SystemBoolean_0 = __lcl_a_SystemInt32_0 != __lcl_b_SystemInt32_0;
        __lcl_gt_SystemBoolean_0 = __lcl_a_SystemInt32_0 > __lcl_b_SystemInt32_0;
        __lcl_gte_SystemBoolean_0 = __lcl_a_SystemInt32_0 >= __lcl_b_SystemInt32_0;
        __lcl_lt_SystemBoolean_0 = __lcl_a_SystemInt32_0 < __lcl_b_SystemInt32_0;
        __lcl_lte_SystemBoolean_0 = __lcl_a_SystemInt32_0 <= __lcl_b_SystemInt32_0;
        return;
    }
}