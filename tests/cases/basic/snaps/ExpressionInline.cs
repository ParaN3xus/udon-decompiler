// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class ExpressionInline : UdonSharpBehaviour
{
    UnityEngine.Vector3 __const_UnityEngineVector3_0 = new UnityEngine.Vector3() /* (0.00, 1.00, 0.00) */;
    UnityEngine.Vector3 __const_UnityEngineVector3_1 = new UnityEngine.Vector3() /* (1.00, 0.00, 0.00) */;

    public void _start()
    {
        System.Int32 __lcl_a_SystemInt32_0;
        System.Int32 __lcl_b_SystemInt32_0;
        System.Single __lcl_f_SystemSingle_0;
        System.Int32 __lcl_res1_SystemInt32_0;
        System.Single __lcl_res2_SystemSingle_0;
        UnityEngine.Vector3 __lcl_v_UnityEngineVector3_0;
        System.Boolean __intnl_SystemBoolean_0;
        System.Boolean __intnl_SystemBoolean_1;
        System.String __lcl_n_SystemString_0;
        __lcl_a_SystemInt32_0 = 10;
        __lcl_b_SystemInt32_0 = 20;
        __lcl_f_SystemSingle_0 = 2.5f;
        __lcl_res1_SystemInt32_0 = (__lcl_a_SystemInt32_0 + __lcl_b_SystemInt32_0) * (__lcl_b_SystemInt32_0 - __lcl_a_SystemInt32_0);
        __lcl_res2_SystemSingle_0 = System.Object.ToSingle(__lcl_a_SystemInt32_0) * 0.5f + __lcl_f_SystemSingle_0 / 2.0f;
        __lcl_v_UnityEngineVector3_0 = __const_UnityEngineVector3_0 * __lcl_f_SystemSingle_0 + __const_UnityEngineVector3_1;
        __intnl_SystemBoolean_1 = __lcl_a_SystemInt32_0 > 5;
        if (__intnl_SystemBoolean_1)
        {
            __intnl_SystemBoolean_1 = __lcl_b_SystemInt32_0 < 30;
        }
        __intnl_SystemBoolean_0 = __intnl_SystemBoolean_1;
        if (__intnl_SystemBoolean_0)
        {
        }
        else
        {
            __intnl_SystemBoolean_0 = __lcl_res1_SystemInt32_0 == 0;
        }
        if (__intnl_SystemBoolean_0)
        {
            __lcl_n_SystemString_0 = this.transform.parent.name;
            UnityEngine.Debug.Log(UnityEngine.Mathf.Abs(__lcl_f_SystemSingle_0).ToString());
        }
        return;
    }
}