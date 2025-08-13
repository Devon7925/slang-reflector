import playground;

[playground::CALL::SIZE_OF("buf")]
[playground::CALL::ONCE]
[shader("compute")]
[numthreads(64, 1, 1)]
void fillBuffer4(uint2 dispatchThreadId : SV_DispatchThreadID)
{
}

[playground::CALL::SIZE_OF("buf")]
[playground::CALL::ONCE]
[shader("compute")]
[numthreads(64, 1, 1)]
void fillBuffer5(uint2 dispatchThreadId : SV_DispatchThreadID)
{
}

[playground::CALL::SIZE_OF("buf")]
[playground::CALL::ONCE]
[shader("compute")]
[numthreads(64, 1, 1)]
void fillBuffer6(uint2 dispatchThreadId : SV_DispatchThreadID)
{
}