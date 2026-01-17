using UdonSharp;
using UnityEngine;
using VRC.SDKBase;
using VRC.Udon;

public class RotatingCube : UdonSharpBehaviour
{
    [SerializeField]
    private float rotationSpeed = 45.0f;

    private void Update()
    {
        transform.Rotate(Vector3.up * rotationSpeed * Time.deltaTime);
    }
}
