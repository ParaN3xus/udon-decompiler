// Decompiled Udon Program
// This is pseudo-code and may not compile directly

public class OutParameter : UdonSharpBehaviour
{
    System.Object[] __gintnl_SystemObjectArray_0 = null /* [null] */;

    public void _start()
    {
        VRC.SDK3.Data.DataDictionary __lcl_myDict_VRCSDK3DataDataDictionary_0;
        VRC.SDK3.Data.DataToken __lcl_outResult_VRCSDK3DataDataToken_0;
        System.Boolean __lcl_success_SystemBoolean_0;
        __lcl_myDict_VRCSDK3DataDataDictionary_0 = new VRC.SDK3.Data.DataDictionary();
        __lcl_myDict_VRCSDK3DataDataDictionary_0.Add((VRC.SDK3.Data.DataToken) "TargetKey", (VRC.SDK3.Data.DataToken)42);
        __lcl_success_SystemBoolean_0 =
            __lcl_myDict_VRCSDK3DataDataDictionary_0.TryGetValue((VRC.SDK3.Data.DataToken) "TargetKey", out __lcl_outResult_VRCSDK3DataDataToken_0);
        if (__lcl_success_SystemBoolean_0)
        {
            __gintnl_SystemObjectArray_0.Set(0, __lcl_outResult_VRCSDK3DataDataToken_0.Int);
            UnityEngine.Debug.LogFormat("Success! Value is: {0}", __gintnl_SystemObjectArray_0);
        }
        else
        {
            UnityEngine.Debug.Log("Key not found.");
        }
        return;
    }
}