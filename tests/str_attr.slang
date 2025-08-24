// Control a uniform with external data
[__AttributeUsage(_AttributeTargets.Struct)]
public struct playground_ANNOTATIONAttribute
{
    string annotation;
};

// A simple struct annotated with playground_ANNOTATION
[playground::ANNOTATION("doc = \"This is a test annotation for the Rust struct generated from Slang sdf.\"")]
struct AnnotatedUniform
{
    uint prop1;
};

RWStructuredBuffer<AnnotatedUniform> outputBuffer;

[shader("compute")]
[numthreads(64, 1, 1)]
void fillBuffer3(uint2 dispatchThreadId: SV_DispatchThreadID)
{
    uint idx = dispatchThreadId.x;
    outputBuffer[idx] = AnnotatedUniform(0);
}