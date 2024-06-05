use bytemuck::{Pod, Zeroable};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

/// ***MUST*** also be `#[repr(C)]`!
pub trait Muck: Pod + Zeroable {}

pub trait WithFormat: Muck {
    const FORMAT: VertexFormat;
}

pub trait WithAttributes: Muck {
    const STEP_MODE: VertexStepMode;
    const ATTRIBUTES: &'static [VertexAttribute];

    fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: Self::STEP_MODE,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

/// ***MUST*** also be `#[repr(C)]`!
#[macro_export]
macro_rules! muck {
    (unsafe $Type:ty => $format:ident) => {
        unsafe impl ::bytemuck::Pod for $Type {}
        unsafe impl ::bytemuck::Zeroable for $Type {}

        impl $crate::muck::Muck for $Type {}

        impl $crate::muck::WithFormat for $Type {
            const FORMAT: ::wgpu::VertexFormat = ::wgpu::VertexFormat::$format;
        }
    };
    (unsafe $Type:ty => $step_mode:ident: [$($WithFormat:ty),* $(,)?]) => {
        unsafe impl ::bytemuck::Pod for $Type {}
        unsafe impl ::bytemuck::Zeroable for $Type {}

        impl $crate::muck::Muck for $Type {}

        impl $crate::muck::WithAttributes for $Type {
            const STEP_MODE: ::wgpu::VertexStepMode = ::wgpu::VertexStepMode::$step_mode;
            const ATTRIBUTES: &'static [::wgpu::VertexAttribute] = $crate::muck!(attributes; $($WithFormat,)*);
        }
    };

    // Terminates
    (attributes; [$($attribute:expr,)*]; $offset:expr; $localion:expr;) => {
        &[$($attribute,)*]
    };
    // Continues
    (attributes; [$($attribute:expr,)*]; $offset:expr; $location:expr; $WithFormat:ty, $($more:ty,)*) => {
        $crate::muck!(
            attributes;
            [$($attribute,)* ::wgpu::VertexAttribute {
                format: <$WithFormat as $crate::muck::WithFormat>::FORMAT,
                offset: $offset,
                shader_location: $location,
            },];
            $offset + <$WithFormat as $crate::muck::WithFormat>::FORMAT.size();
            $location + 1;
            $($more,)*
        )
    };
    // Starts
    (attributes; $($WithFormat:ty,)*) => {
        $crate::muck!(attributes; []; 0; 0; $($WithFormat,)*)
    };
}
