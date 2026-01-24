// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class IfElse : UdonSharpBehaviour
{
    public void _start()
    {
        System.Int32 __lcl_score_SystemInt32_0;
        System.String __lcl_result_SystemString_0;
        __lcl_score_SystemInt32_0 = 75;
        if (__lcl_score_SystemInt32_0 >= 60)
        {
            __lcl_result_SystemString_0 = "Pass";
            UnityEngine.Debug.Log("Good job");
        }
        else
        {
            __lcl_result_SystemString_0 = "Fail";
            UnityEngine.Debug.Log("Try again");
        }
        UnityEngine.Debug.Log(__lcl_result_SystemString_0);
        return;
    }
}