using UnityEngine;
using UnityEditor;

public class UdonSharpSourceTextCompilerGUI : EditorWindow
{
    [SerializeField] private string className = "TempSourceText";

    [SerializeField] private string sourceCode = @"
using UnityEngine;
using UdonSharp;

public class TempSourceText : UdonSharpBehaviour
{
    public override void Interact()
    {
        Debug.Log(""Hello World"");
    }
}";
    [SerializeField] private string jsonOutput = "";
    private Vector2 scrollPosition;

    [MenuItem("Tools/Udon Source Compiler")]
    public static void ShowWindow()
    {
        GetWindow<UdonSharpSourceTextCompilerGUI>("Udon Compiler");
    }

    private void OnEnable()
    {
        UdonSharpSourceTextCompiler.OnDumpCompleted += OnCompilationFinished;
    }

    private void OnDisable()
    {
        UdonSharpSourceTextCompiler.OnDumpCompleted -= OnCompilationFinished;
    }

    private void OnCompilationFinished(string jsonResult)
    {
        jsonOutput = jsonResult;
        Repaint();

        this.Focus();
    }

    private void OnGUI()
    {
        GUIStyle areaStyle = new GUIStyle(EditorStyles.textArea);
        areaStyle.wordWrap = true;

        scrollPosition = EditorGUILayout.BeginScrollView(scrollPosition);

        GUILayout.Label("Settings:", EditorStyles.boldLabel);

        className = EditorGUILayout.TextField("Class Name", className);

        GUILayout.Space(10);

        GUILayout.Label("Input Udon Assembly Code:", EditorStyles.boldLabel);

        sourceCode = EditorGUILayout.TextArea(sourceCode, areaStyle, GUILayout.MinHeight(150), GUILayout.ExpandHeight(true));

        GUILayout.Space(10);

        if (GUILayout.Button("Compile and Dump", GUILayout.Height(40)))
        {
            if (string.IsNullOrEmpty(className))
            {
                Debug.LogError("Class Name cannot be empty.");
                return;
            }

            jsonOutput = "Compiling... Please wait for Unity Domain Reload.\n(System will freeze briefly)";

            UdonSharpSourceTextCompiler.CompileAndDump(sourceCode, className);
        }

        GUILayout.Space(10);

        GUILayout.Label("Dumped JSON:", EditorStyles.boldLabel);

        jsonOutput = EditorGUILayout.TextArea(jsonOutput, areaStyle, GUILayout.MinHeight(150), GUILayout.ExpandHeight(true));

        EditorGUILayout.EndScrollView();
    }
}
