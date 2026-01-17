using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class HelloWorld : UdonSharpBehaviour
{
    public override void Interact()
    {
        Debug.Log("Hello from Udon! You clicked me.");
    }
}
