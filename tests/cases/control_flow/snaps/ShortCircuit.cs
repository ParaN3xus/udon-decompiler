// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class ShortCircuit : UdonSharpBehaviour
{
    System.Int32 a = 10;
    System.Int32 b = 20;
    System.Int32 c = 30;

    public void _interact()
    {
        System.Boolean __intnl_SystemBoolean_0 = false;
        System.Boolean __intnl_SystemBoolean_1 = false;
        System.Boolean __intnl_SystemBoolean_2 = false;
        System.Boolean __intnl_SystemBoolean_3 = false;

        if (a > 0)
        {
            __intnl_SystemBoolean_0 = b < 100;
        }
        if (__intnl_SystemBoolean_0)
        {
            UnityEngine.Debug.Log("AND Check Passed");
        }
        if (!(a == 5))
        {
            __intnl_SystemBoolean_1 = b == 5;
        }
        if (__intnl_SystemBoolean_1)
        {
            UnityEngine.Debug.Log("OR Check Passed");
        }
        if (a > b)
        {
            if (!(b > c))
            {
                __intnl_SystemBoolean_3 = a == 10;
            }
            __intnl_SystemBoolean_2 = __intnl_SystemBoolean_3;
        }
        if (__intnl_SystemBoolean_2)
        {
            UnityEngine.Debug.Log("Mixed Check Passed");
        }
        return;
    }
}