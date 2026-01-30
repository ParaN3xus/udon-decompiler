// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class VariableScope : UdonSharpBehaviour
{
    System.Int32 fieldVal = 10;

    public void _start()
    {
        System.Int32 __lcl_rootVar_SystemInt32_0;
        System.Int32 __lcl_i_SystemInt32_0;
        System.Int32 __lcl_branchVar_SystemInt32_0;
        System.Int32 __lcl_loopInternal_SystemInt32_0;
        System.Int32 __lcl_deepVar_SystemInt32_0;
        __lcl_rootVar_SystemInt32_0 = 100;
        if (fieldVal > 5)
        {
            __lcl_branchVar_SystemInt32_0 = fieldVal * 2;
            __lcl_rootVar_SystemInt32_0 = __lcl_rootVar_SystemInt32_0 + __lcl_branchVar_SystemInt32_0;
        }
        else
        {
            __lcl_branchVar_SystemInt32_0 = 0;
            __lcl_rootVar_SystemInt32_0 = __lcl_rootVar_SystemInt32_0 - __lcl_branchVar_SystemInt32_0;
        }
        __lcl_i_SystemInt32_0 = 0;
        while (__lcl_i_SystemInt32_0 < 5)
        {
            __lcl_loopInternal_SystemInt32_0 = __lcl_i_SystemInt32_0 * 10;
            __lcl_rootVar_SystemInt32_0 = __lcl_rootVar_SystemInt32_0 + __lcl_loopInternal_SystemInt32_0;
            if (__lcl_loopInternal_SystemInt32_0 > 20)
            {
                __lcl_deepVar_SystemInt32_0 = 1;
                __lcl_rootVar_SystemInt32_0 = __lcl_rootVar_SystemInt32_0 + __lcl_deepVar_SystemInt32_0;
            }
            __lcl_i_SystemInt32_0 = __lcl_i_SystemInt32_0 + 1;
        }
        fieldVal = __lcl_rootVar_SystemInt32_0;
        return;
    }
}