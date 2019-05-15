﻿using Template;
using Cloo;
using System;

namespace clrays
{
    public interface VoidKernel<T> where T : struct
    {
        void Execute(ComputeEventList events);
        OpenCLBuffer<T> GetBuffer();
    }

    public interface ResultKernel<T> where T: struct
    {
        void Execute(ComputeEventList events);
        T[] GetResult();
        OpenCLBuffer<T> GetBuffer();
    }

    public class TraceAaKernel : ResultKernel<float>
    {
        private OpenCLKernel kernel;
        private readonly long[] work;
        private readonly float[] data;
        private readonly OpenCLBuffer<float> buffer;
        private readonly OpenCLBuffer<int> scene_params;
        private readonly OpenCLBuffer<float> scene_items;
        private bool dirty;

        public TraceAaKernel(OpenCLProgram program, Scene scene, uint width, uint height, uint AA)
        {
            data = new float[width * height * 3];
            buffer = new OpenCLBuffer<float>(program, data);
            //make kernel and set args
            kernel = new OpenCLKernel(program, "render");
            kernel.SetArgument(0, buffer);
            //make scene buffers
            var scene_raw = scene.GetBuffers();
            var scene_params_raw = scene.GetParamsBuffer();
            scene_params = new OpenCLBuffer<int>(program, scene_params_raw);
            scene_items = new OpenCLBuffer<float>(program, scene_raw);
            //set constants
            kernel.SetArgument(1, width);
            kernel.SetArgument(2, height);
            kernel.SetArgument(3, AA);
            //set arrays
            kernel.SetArgument(4, scene_params);
            kernel.SetArgument(5, scene_items);
            //work
            work = new long[] { width * AA, height * AA };
            dirty = false;
        }

        public void Execute(ComputeEventList events)
        {
            kernel.Execute(work, events);
            dirty = true;
        }

        public float[] GetResult()
        {
            if(dirty)
                buffer.CopyFromDevice();
            dirty = false;
            return data;
        }

        public OpenCLBuffer<float> GetBuffer()
        {
            return buffer;
        }
    }

    public class ClearKernel : VoidKernel<float>
    {
        private OpenCLKernel kernel;
        private readonly OpenCLBuffer<float> buffer;
        private readonly long[] work;

        public ClearKernel(OpenCLProgram program, OpenCLBuffer<float> buffer, uint width, uint height)
        {
            kernel = new OpenCLKernel(program, "clear");
            kernel.SetArgument(0, buffer);
            kernel.SetArgument(1, width);
            kernel.SetArgument(2, height);
            work = new long[] { width, height };
            this.buffer = buffer;
        }

        public void Execute(ComputeEventList events)
        {
            kernel.Execute(work, events);
        }

        public OpenCLBuffer<float> GetBuffer()
        {
            return buffer;
        }
    }

    public class ImageKernel : ResultKernel<int>
    {
        private OpenCLKernel kernel;
        private readonly long[] work;
        private readonly int[] data;
        private OpenCLBuffer<int> buffer;
        private bool dirty;

        public ImageKernel(OpenCLProgram program, OpenCLBuffer<float> input, uint width, uint height)
        {
            data = new int[width * height];
            buffer = new OpenCLBuffer<int>(program, data);
            kernel = new OpenCLKernel(program, "image_from_floatmap");
            kernel.SetArgument(0, input);
            kernel.SetArgument(1, buffer);
            kernel.SetArgument(2, width);
            kernel.SetArgument(3, height);
            work = new long[] { width, height };
            dirty = false;
        }

        public void Execute(ComputeEventList events)
        {
            kernel.Execute(work, events);
            dirty = true;
        }

        public int[] GetResult()
        {
            if (dirty)
                buffer.CopyFromDevice();
            dirty = false;
            return data;
        }

        public OpenCLBuffer<int> GetBuffer()
        {
            return buffer;
        }
    }
}
