struct VSIn
{
    float3 position : POSITION;
};

float4 VS(VSIn input) : SV_Position
{
    float4 pos = float4(input.position, 1.0f);

    //pos.z += 5.0f;

    return pos;
}

float4 PS() : SV_Target
{
    return float4(0.8f, 0.8f, 0.3f, 1.0f);
}
