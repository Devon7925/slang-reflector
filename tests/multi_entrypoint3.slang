import playground;

[playground::CALL::SIZE_OF("buf")]
[playground::CALL::ONCE]
[shader("compute")]
[numthreads(64, 1, 1)]
void fillBuffer7(uint2 dispatchThreadId : SV_DispatchThreadID)
{
}

[playground::CALL::SIZE_OF("buf")]
[playground::CALL::ONCE]
[shader("compute")]
[numthreads(64, 1, 1)]
void fillBuffer8(uint2 dispatchThreadId : SV_DispatchThreadID)
{
}

[playground::CALL::SIZE_OF("buf")]
[playground::CALL::ONCE]
[shader("compute")]
[numthreads(64, 1, 1)]
void fillBuffer9(uint2 dispatchThreadId : SV_DispatchThreadID)
{
}