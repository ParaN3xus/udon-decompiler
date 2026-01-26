// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class ShortCircuit : UdonSharpBehaviour
{
    System.Int32 a = 10;
    System.Int32 b = 20;
    System.Int32 c = 30;

    public void _interact()
    {
        System.Boolean __intnl_SystemBoolean_0;
        System.Boolean __intnl_SystemBoolean_1;
        System.Boolean __intnl_SystemBoolean_2;
        System.Boolean __intnl_SystemBoolean_3;
        __intnl_SystemBoolean_0 = a > 0;
        if (__intnl_SystemBoolean_0)
        {
            __intnl_SystemBoolean_0 = b < 100;
        }
        if (__intnl_SystemBoolean_0)
        {
            UnityEngine.Debug.Log("AND Check Passed");
        }
        __intnl_SystemBoolean_1 = a == 5;
        if (!__intnl_SystemBoolean_1)
        {
            __intnl_SystemBoolean_1 = b == 5;
        }
        if (__intnl_SystemBoolean_1)
        {
            UnityEngine.Debug.Log("OR Check Passed");
        }
        __intnl_SystemBoolean_2 = a > b;
        if (__intnl_SystemBoolean_2)
        {
            __intnl_SystemBoolean_3 = b > c;
            if (!__intnl_SystemBoolean_3)
            {
                __intnl_SystemBoolean_3 = a == 10;
            }
            __intnl_SystemBoolean_2 = __intnl_SystemBoolean_3;
            if (__intnl_SystemBoolean_2)
            {
                UnityEngine.Debug.Log("Mixed Check Passed");
            }
            return;
        }
    }
}