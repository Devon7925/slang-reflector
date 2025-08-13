import playground;
import multi_entrypoint2;
import multi_entrypoint3;

[playground::TIME]
uniform float time;

[playground::RAND(131072)]
RWStructuredBuffer<float> buf;

[playground::CALL::SIZE_OF("buf")]
[playground::CALL::ONCE]
[shader("compute")]
[numthreads(64, 1, 1)]
void fillBuffer(uint2 dispatchThreadId : SV_DispatchThreadID)
{
    uint idx = dispatchThreadId.x;
    buf[idx] = sin(float(idx) / 1000.0 + 10 * time);
}

[playground::CALL::SIZE_OF("buf")]
[playground::CALL::ONCE]
[shader("compute")]
[numthreads(64, 1, 1)]
void fillBuffer2(uint2 dispatchThreadId : SV_DispatchThreadID)
{
    uint idx = dispatchThreadId.x;
    buf[idx] = sin(float(idx) / 1000.0 + 10 * time);
}

[playground::CALL::SIZE_OF("buf")]
[playground::CALL::ONCE]
[shader("compute")]
[numthreads(64, 1, 1)]
void fillBuffer3(uint2 dispatchThreadId : SV_DispatchThreadID)
{
    uint idx = dispatchThreadId.x;
    buf[idx] = sin(float(idx) / 1000.0 + 10 * time);
}

float4 imageMain(uint2 dispatchThreadID, int2 screenSize)
{
    uint idx = dispatchThreadID.x + dispatchThreadID.y * screenSize.x;

    float rand = buf[idx % 131072];

    return float4(rand, rand, rand, 1.0);
}