// Tensor API and model exchange (Spec §60).
//
// Portable core: row-major f32 tensors with shape-checked matmul,
// transpose and reductions. GPU (CUDA/ROCm/OpenCL) and runtime
// integrations (ONNX/TensorRT) are bindings behind this API so the core
// language stays portable. NumPy `.npy` v1.0 (C-order f32) import/export
// provides the Python-interop bridge.

#[derive(Debug, Clone, PartialEq)]
pub struct Tensor {
    pub shape: Vec<usize>,
    pub data: Vec<f32>,
}

impl Tensor {
    /// Row-major tensor; `data.len()` must equal the shape product.
    pub fn new(shape: Vec<usize>, data: Vec<f32>) -> Option<Self> {
        let expect: usize = shape.iter().product();
        if data.len() != expect {
            return None;
        }
        Some(Tensor { shape, data })
    }

    pub fn zeros(shape: Vec<usize>) -> Self {
        let n: usize = shape.iter().product();
        Tensor { shape, data: vec![0.0; n] }
    }

    pub fn rank(&self) -> usize {
        self.shape.len()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn idx2(&self, r: usize, c: usize) -> Option<usize> {
        if self.rank() != 2 || r >= self.shape[0] || c >= self.shape[1] {
            return None;
        }
        Some(r * self.shape[1] + c)
    }

    pub fn get2(&self, r: usize, c: usize) -> Option<f32> {
        self.idx2(r, c).map(|i| self.data[i])
    }

    /// Matrix multiplication for 2-D tensors with full shape checking.
    pub fn matmul(&self, rhs: &Tensor) -> Option<Tensor> {
        if self.rank() != 2 || rhs.rank() != 2 {
            return None;
        }
        let (m, k) = (self.shape[0], self.shape[1]);
        let (k2, n) = (rhs.shape[0], rhs.shape[1]);
        if k != k2 {
            return None;
        }
        let mut out = vec![0.0f32; m * n];
        for i in 0..m {
            for j in 0..n {
                let mut acc = 0.0f32;
                for p in 0..k {
                    acc += self.data[i * k + p] * rhs.data[p * n + j];
                }
                out[i * n + j] = acc;
            }
        }
        Tensor::new(vec![m, n], out)
    }

    pub fn transpose2(&self) -> Option<Tensor> {
        if self.rank() != 2 {
            return None;
        }
        let (r, c) = (self.shape[0], self.shape[1]);
        let mut out = vec![0.0f32; r * c];
        for i in 0..r {
            for j in 0..c {
                out[j * r + i] = self.data[i * c + j];
            }
        }
        Tensor::new(vec![c, r], out)
    }

    pub fn sum(&self) -> f32 {
        self.data.iter().sum()
    }

    // -- Part X / §60: element-wise portable ops (SIMD-friendly) -----

    /// Element-wise addition with exact shape equality.
    pub fn add_elem(&self, rhs: &Tensor) -> Option<Tensor> {
        if self.shape != rhs.shape {
            return None;
        }
        Tensor::new(
            self.shape.clone(),
            self.data.iter().zip(rhs.data.iter()).map(|(a, b)| a + b).collect(),
        )
    }

    /// Element-wise `max(0, x)` (inference activation).
    pub fn relu(&self) -> Tensor {
        Tensor {
            shape: self.shape.clone(),
            data: self.data.iter().map(|x| x.max(0.0)).collect(),
        }
    }

    /// Numerically stable softmax over the flat data.
    pub fn softmax(&self) -> Tensor {
        let max = self.data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let exps: Vec<f32> = self.data.iter().map(|x| (x - max).exp()).collect();
        let sum: f32 = exps.iter().sum();
        let data = if sum == 0.0 { exps } else { exps.iter().map(|x| x / sum).collect() };
        Tensor { shape: self.shape.clone(), data }
    }

    /// Index of the maximum element (classification decision).
    pub fn argmax(&self) -> Option<usize> {
        if self.data.is_empty() {
            return None;
        }
        let mut best = 0;
        for i in 1..self.data.len() {
            if self.data[i] > self.data[best] {
                best = i;
            }
        }
        Some(best)
    }

    /// Fused linear layer: `self @ weights + bias`.
    /// 2-D path: `(m,k) @ (k,n) + (n,)` broadcast over rows.
    /// 1-D path: `(k,) @ (k,n) + (n,)` row-vector → `(n,)`.
    /// Ownership: returns owned tensor. Thread-safety: `Send + Sync`.
    /// Complexity: O(m*k*n).
    pub fn linear(&self, weights: &Tensor, bias: &[f32]) -> Option<Tensor> {
        if self.rank() == 1 {
            if weights.rank() != 2 {
                return None;
            }
            let (k, n) = (self.shape[0], weights.shape[1]);
            if weights.shape[0] != k || bias.len() != n {
                return None;
            }
            let mut out = vec![0.0f32; n];
            for j in 0..n {
                let mut acc = 0.0f32;
                for i in 0..k {
                    acc += self.data[i] * weights.data[i * n + j];
                }
                out[j] = acc + bias[j];
            }
            return Tensor::new(vec![n], out);
        }
        let mut out = self.matmul(weights)?;
        if bias.len() != out.shape[1] {
            return None;
        }
        let n = out.shape[1];
        for (i, v) in out.data.iter_mut().enumerate() {
            *v += bias[i % n];
        }
        Some(out)
    }

    // -- NumPy .npy v1.0 bridge (C-order, f32 only) ---------------------

    /// Serialize to NumPy `.npy` v1.0 bytes.
    pub fn to_npy(&self) -> Vec<u8> {
        let shape_str = self.shape.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", ");
        let shape_repr = if self.shape.len() == 1 {
            format!("({},)", shape_str)
        } else {
            format!("({})", shape_str)
        };
        let mut header = format!(
            "{{'descr': '<f4', 'fortran_order': False, 'shape': {}, }}",
            shape_repr
        );
        header.push('\n');
        // Pad so that magic(6) + ver(2) + len(2) + header is 64-aligned.
        let pre = 10 + header.len();
        let pad = (64 - (pre % 64)) % 64;
        header.extend(std::iter::repeat(' ').take(pad));
        let mut out = Vec::new();
        out.extend_from_slice(b"\x93NUMPY");
        out.extend_from_slice(&[1, 0]);
        out.extend_from_slice(&(header.len() as u16).to_le_bytes());
        out.extend_from_slice(header.as_bytes());
        for f in &self.data {
            out.extend_from_slice(&f.to_le_bytes());
        }
        out
    }

    /// Parse NumPy `.npy` v1.0 C-order f32 arrays.
    pub fn from_npy(bytes: &[u8]) -> Option<Tensor> {
        if bytes.len() < 10 || &bytes[0..6] != b"\x93NUMPY" {
            return None;
        }
        if bytes[6] != 1 || bytes[7] != 0 {
            return None; // only v1.0
        }
        let hlen = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
        if bytes.len() < 10 + hlen {
            return None;
        }
        let header = std::str::from_utf8(&bytes[10..10 + hlen]).ok()?;
        // Require little-endian f32, C order.
        if !header.contains("<f4") || !header.contains("'fortran_order': False") {
            return None;
        }
        let shape = parse_npy_shape(header)?;
        let count: usize = shape.iter().product();
        let body = &bytes[10 + hlen..];
        if body.len() < count * 4 {
            return None;
        }
        let mut data = Vec::with_capacity(count);
        for chunk in body[..count * 4].chunks_exact(4) {
            data.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
        Tensor::new(shape, data)
    }
}

// ---------------------------------------------------------------------------
// Part X / §60–§61: GPU stubs (Buffer/Device) + inference stub.
//
// Portability rule (Spec §60): GPU functionality MUST NOT compromise
// the portability of the core language. These types expose device
// selection, buffer ownership and model inference SHAPES; actual
// CUDA/ROCm/OpenCL/ONNX/TensorRT execution lives in backend bindings.
// The portable CPU fallback (`InferenceSession::infer_cpu`) always
// works, so tests and host builds never require a GPU.
// ---------------------------------------------------------------------------

/// GPU compute backend (binding target, never assumed present).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuBackend {
    Cuda,
    Rocm,
    OpenCl,
    Wgpu,
    /// Portable CPU fallback (always available).
    Cpu,
}

impl GpuBackend {
    pub fn name(self) -> &'static str {
        match self {
            GpuBackend::Cuda => "cuda",
            GpuBackend::Rocm => "rocm",
            GpuBackend::OpenCl => "opencl",
            GpuBackend::Wgpu => "wgpu",
            GpuBackend::Cpu => "cpu",
        }
    }

    /// Whether this backend *might* be usable on this host. The stub
    /// reports `Cpu` as available and everything else as unavailable
    /// until a real binding registers itself.
    pub fn is_available(self) -> bool {
        matches!(self, GpuBackend::Cpu)
    }
}

/// Logical GPU device handle (stub: no hardware touched).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuDevice {
    pub backend: GpuBackend,
    pub index: u32,
    pub name: String,
}

impl GpuDevice {
    pub fn cpu() -> Self {
        GpuDevice { backend: GpuBackend::Cpu, index: 0, name: "cpu-fallback".to_string() }
    }

    pub fn stub(backend: GpuBackend, index: u32) -> Self {
        GpuDevice { backend, index, name: format!("{}:{}", backend.name(), index) }
    }

    pub fn is_usable(&self) -> bool {
        self.backend.is_available()
    }
}

/// Device buffer descriptor (stub: owns a CPU mirror; device upload is
/// a binding concern). Ownership: the buffer owns `mirror`; `upload`
/// / `download` are explicit copies — no aliasing across the boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct GpuBuffer {
    pub device: GpuDevice,
    pub len_bytes: usize,
    pub id: u64,
    mirror: Vec<u8>,
}

impl GpuBuffer {
    pub fn allocate(device: GpuDevice, len_bytes: usize, id: u64) -> Self {
        GpuBuffer { device, len_bytes, id, mirror: vec![0u8; len_bytes] }
    }

    /// Stage host bytes into the mirror (bounds-checked).
    pub fn upload(&mut self, bytes: &[u8]) -> Result<(), String> {
        if bytes.len() != self.len_bytes {
            return Err(format!(
                "E0501: gpu upload size mismatch: buffer {} bytes, got {}",
                self.len_bytes,
                bytes.len()
            ));
        }
        self.mirror.copy_from_slice(bytes);
        Ok(())
    }

    pub fn download(&self) -> &[u8] {
        &self.mirror
    }

    /// Copy a tensor's raw f32 bytes into the buffer.
    pub fn upload_tensor(&mut self, t: &Tensor) -> Result<(), String> {
        let mut bytes = Vec::with_capacity(t.data.len() * 4);
        for f in &t.data {
            bytes.extend_from_slice(&f.to_le_bytes());
        }
        self.upload(&bytes)
    }
}

/// Minimal model descriptor for inference plumbing.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub name: String,
    pub input_shape: Vec<usize>,
    pub output_shape: Vec<usize>,
}

impl Model {
    pub fn new(name: impl Into<String>, input_shape: Vec<usize>, output_shape: Vec<usize>) -> Self {
        Model { name: name.into(), input_shape, output_shape }
    }
}

/// Single-layer dense model used by the portable inference stub:
/// `softmax(relu(x @ weights + bias))`.
#[derive(Debug, Clone, PartialEq)]
pub struct InferenceSession {
    pub model: Model,
    pub weights: Tensor,
    pub bias: Vec<f32>,
}

impl InferenceSession {
    /// Build a session, checking `weights`/`bias` against the model shapes.
    pub fn load(model: Model, weights: Tensor, bias: Vec<f32>) -> Option<Self> {
        if weights.rank() != 2 {
            return None;
        }
        let in_elems: usize = model.input_shape.iter().product();
        let out_elems: usize = model.output_shape.iter().product();
        if weights.shape[0] != in_elems || weights.shape[1] != out_elems {
            return None;
        }
        if bias.len() != out_elems {
            return None;
        }
        Some(InferenceSession { model, weights, bias })
    }

    /// Portable CPU inference (always available): validates the input
    /// shape, then runs the dense layer. Returns `None` on shape mismatch.
    pub fn infer_cpu(&self, input: &Tensor) -> Option<Tensor> {
        if input.shape != self.model.input_shape && !(input.rank() == 2 && input.shape[1..] == self.model.input_shape[1..]) {
            // Accept row-batched inputs whose trailing dims match.
            if input.shape.len() != self.model.input_shape.len() {
                return None;
            }
        }
        // Flatten leading batch into rows of `in_elems`.
        let in_elems: usize = self.model.input_shape.iter().product();
        if input.data.len() % in_elems != 0 {
            return None;
        }
        let rows = input.data.len() / in_elems;
        let flat_in = Tensor::new(vec![rows, in_elems], input.data.clone())?;
        let logits = flat_in.linear(&self.weights, &self.bias)?;
        // Row-wise softmax.
        let n = logits.shape[1];
        let mut data = Vec::with_capacity(logits.data.len());
        for row in logits.data.chunks(n) {
            let t = Tensor::new(vec![n], row.to_vec())?;
            data.extend_from_slice(&t.softmax().data);
        }
        Tensor::new(vec![rows, n], data)
    }

    /// Device inference entry point (stub): routes to the CPU fallback
    /// with a note when the device is unavailable, so callers observe
    /// identical numerics without hardware.
    pub fn infer(&self, device: &GpuDevice, input: &Tensor) -> (Option<Tensor>, &'static str) {
        if device.is_usable() {
            (self.infer_cpu(input), "cpu")
        } else {
            (self.infer_cpu(input), "cpu-fallback (device unavailable)")
        }
    }
}

fn parse_npy_shape(header: &str) -> Option<Vec<usize>> {
    let start = header.find("'shape'")?;
    let after = &header[start..];
    let l = after.find('(')?;
    let r = after.find(')')?;
    let inner = after[l + 1..r].trim();
    if inner.is_empty() {
        return Some(vec![]);
    }
    inner
        .split(',')
        .filter_map(|p| {
            let p = p.trim();
            if p.is_empty() {
                None
            } else {
                p.parse::<usize>().ok()
            }
        })
        .collect::<Vec<_>>()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matmul_shape_checked() {
        let a = Tensor::new(vec![2, 3], vec![1., 2., 3., 4., 5., 6.]).unwrap();
        let b = Tensor::new(vec![3, 2], vec![7., 8., 9., 10., 11., 12.]).unwrap();
        let c = a.matmul(&b).unwrap();
        assert_eq!(c.shape, vec![2, 2]);
        assert_eq!(c.data, vec![58., 64., 139., 154.]);
        assert!(a.matmul(&a).is_none());
        assert!(Tensor::new(vec![2, 2], vec![1.]).is_none());
    }

    #[test]
    fn transpose_and_sum() {
        let a = Tensor::new(vec![2, 3], vec![1., 2., 3., 4., 5., 6.]).unwrap();
        let t = a.transpose2().unwrap();
        assert_eq!(t.shape, vec![3, 2]);
        assert_eq!(t.get2(1, 0), Some(2.0));
        assert_eq!(a.sum(), 21.0);
    }

    #[test]
    fn npy_roundtrip() {
        let a = Tensor::new(vec![2, 3], vec![1., 2., 3., 4., 5., 6.]).unwrap();
        let bytes = a.to_npy();
        let b = Tensor::from_npy(&bytes).unwrap();
        assert_eq!(a, b);
        assert!(Tensor::from_npy(b"garbage").is_none());
    }

    #[test]
    fn portable_ops() {
        let a = Tensor::new(vec![3], vec![-1., 2., -3.]).unwrap();
        assert_eq!(a.relu().data, vec![0., 2., 0.]);
        let b = Tensor::new(vec![2], vec![1., 1.]).unwrap();
        let s = b.softmax();
        assert!((s.data[0] - 0.5).abs() < 1e-6);
        assert_eq!(b.argmax(), Some(0));
        let c = Tensor::new(vec![2], vec![1., 2.]).unwrap();
        assert_eq!(b.add_elem(&c).unwrap().data, vec![2., 3.]);
        assert!(b.add_elem(&a).is_none());
        let w = Tensor::new(vec![2, 1], vec![1., 1.]).unwrap();
        let lin = b.linear(&w, &[0.5]).unwrap();
        assert_eq!(lin.data, vec![2.5]);
        assert!(b.linear(&w, &[0.5, 1.0]).is_none());
    }

    #[test]
    fn gpu_buffer_upload_and_inference_stub() {
        let dev = GpuDevice::cpu();
        assert!(dev.is_usable());
        let cuda = GpuDevice::stub(GpuBackend::Cuda, 0);
        assert!(!cuda.is_usable());
        let t = Tensor::new(vec![2], vec![1., 2.]).unwrap();
        let mut buf = GpuBuffer::allocate(dev.clone(), 8, 1);
        buf.upload_tensor(&t).unwrap();
        assert_eq!(buf.download().len(), 8);
        assert!(buf.upload(&[0u8; 3]).is_err());

        let model = Model::new("tiny", vec![2], vec![2]);
        let weights = Tensor::new(vec![2, 2], vec![1., 0., 0., 1.]).unwrap();
        let sess = InferenceSession::load(model, weights, vec![0., 0.]).unwrap();
        let (out, path) = sess.infer(&dev, &t);
        let out = out.unwrap();
        assert_eq!(out.shape, vec![1, 2]);
        assert!((out.data.iter().sum::<f32>() - 1.0).abs() < 1e-5);
        assert_eq!(path, "cpu");
        let (out2, note) = sess.infer(&cuda, &t);
        assert!(out2.is_some());
        assert!(note.contains("fallback"));
        assert!(sess.infer_cpu(&Tensor::zeros(vec![3])).is_none());
    }
}
