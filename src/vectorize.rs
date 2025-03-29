use crate::gen_ir::{Function, IROp, IR};

// Detects if a loop can be vectorized
fn can_vectorize_loop(ir: &[IR]) -> bool {
    // Simple heuristic: look for loops with regular memory access patterns
    // and simple arithmetic operations
    
    // Check for loop pattern
    let mut has_loop = false;
    let mut has_regular_access = false;
    let mut has_simple_arithmetic = false;
    
    for i in 0..ir.len() {
        // Look for loop structure (Label followed by conditional jump back to label)
        if let IROp::Label = ir[i].op {
            // Check if there's a jump back to this label
            for j in i+1..ir.len() {
                if let IROp::Jmp = ir[j].op {
                    if ir[j].lhs == ir[i].lhs {
                        has_loop = true;
                        break;
                    }
                }
            }
        }
        
        // Look for regular memory access (consecutive loads/stores)
        if let IROp::Load(_) = ir[i].op {
            if i > 0 && i < ir.len() - 1 {
                if let IROp::Load(_) = ir[i-1].op {
                    if let IROp::Load(_) = ir[i+1].op {
                        has_regular_access = true;
                    }
                }
            }
        }
        
        // Look for simple arithmetic operations
        if matches!(ir[i].op, IROp::Add | IROp::Sub | IROp::Mul | IROp::Div) {
            has_simple_arithmetic = true;
        }
    }
    
    has_loop && has_regular_access && has_simple_arithmetic
}

// Identify array operations that can be vectorized
fn identify_array_operations(ir: &[IR]) -> Vec<(usize, usize)> {
    let mut vector_ops = Vec::new();
    let mut i = 0;
    
    while i < ir.len() {
        // Look for patterns like:
        // 1. Load from array
        // 2. Perform arithmetic
        // 3. Store back to array
        
        let start_idx = i;
        let mut is_vector_op = false;
        let mut end_idx = i;
        
        // Check for load operation
        if i < ir.len() && matches!(ir[i].op, IROp::Load(_)) {
            i += 1;
            
            // Check for arithmetic operation
            if i < ir.len() && matches!(ir[i].op, IROp::Add | IROp::Sub | IROp::Mul | IROp::Div) {
                i += 1;
                
                // Check for store operation
                if i < ir.len() && matches!(ir[i].op, IROp::Store(_)) {
                    is_vector_op = true;
                    end_idx = i;
                    i += 1;
                }
            }
        }
        
        if is_vector_op {
            vector_ops.push((start_idx, end_idx));
        } else {
            i += 1;
        }
    }
    
    vector_ops
}

// Detect reduction patterns (sum, min, max)
fn detect_reduction_patterns(ir: &[IR]) -> Vec<(usize, usize, IROp)> {
    let mut reductions = Vec::new();
    let mut i = 0;
    
    while i < ir.len() - 2 {
        // Look for load followed by add/min/max and store to same variable
        if matches!(ir[i].op, IROp::Load(_)) {
            let load_dst = ir[i].lhs;
            let load_src = ir[i].rhs;
            
            if i + 1 < ir.len() {
                let op = match ir[i+1].op {
                    IROp::Add => Some(IROp::AVX512Add),
                    IROp::Mul => Some(IROp::AVX512Mul),
                    // Add more reduction operations
                    _ => None
                };
                
                if let Some(reduction_op) = op {
                    if i + 2 < ir.len() && matches!(ir[i+2].op, IROp::Store(_)) {
                        if ir[i+2].rhs == load_dst && ir[i+2].lhs == load_src {
                            // Found a reduction pattern
                            reductions.push((i, i+2, reduction_op));
                        }
                    }
                }
            }
        }
        
        i += 1;
    }
    
    reductions
}

// Convert regular operations to SIMD operations with more intelligence
fn convert_to_avx512(ir: &mut [IR]) {
    for i in 0..ir.len() {
        match ir[i].op {
            // Convert floating-point operations
            IROp::Add => ir[i].op = IROp::AVX512Add,
            IROp::Sub => ir[i].op = IROp::AVX512Sub,
            IROp::Mul => {
                if i + 1 < ir.len() && matches!(ir[i+1].op, IROp::Add) {
                    if ir[i].lhs == ir[i+1].lhs {
                        // Convert to FMA
                        ir[i].op = IROp::AVX512FMA;
                        ir[i+1].op = IROp::Nop; // Remove the add operation
                    } else {
                        ir[i].op = IROp::AVX512Mul;
                    }
                } else {
                    ir[i].op = IROp::AVX512Mul;
                }
            },
            IROp::Div => ir[i].op = IROp::AVX512Div,
            
            // Convert integer operations
            IROp::AddImm => ir[i].op = IROp::AVX512Addi,
            IROp::SubImm => ir[i].op = IROp::AVX512Subi,
            IROp::MulImm => ir[i].op = IROp::AVX512Muli,
            
            // Convert memory operations
            IROp::Load(_) => {
                // Determine if we're loading integers or floats
                // This is a simplification - in a real implementation, you'd check the type
                ir[i].op = IROp::AVX512Load;
            },
            IROp::Store(_) => {
                // Determine if we're storing integers or floats
                // This is a simplification - in a real implementation, you'd check the type
                ir[i].op = IROp::AVX512Store;
            },
            IROp::Mov => ir[i].op = IROp::AVX512Mov,
            
            // Convert comparison operations
            IROp::LT => ir[i].op = IROp::AVX512Cmplt,
            IROp::LE => ir[i].op = IROp::AVX512Cmple,
            IROp::EQ => ir[i].op = IROp::AVX512Cmpeq,
            
            // Add more pattern-specific conversions
            _ => {}
        }
    }
}

// Apply vectorization to specific ranges of IR instructions
fn vectorize_ranges(ir: &mut [IR], ranges: &[(usize, usize)]) {
    for &(start, end) in ranges {
        for i in start..=end {
            match ir[i].op {
                IROp::Add => ir[i].op = IROp::AVX512Add,
                IROp::Sub => ir[i].op = IROp::AVX512Sub,
                IROp::Mul => ir[i].op = IROp::AVX512Mul,
                IROp::Div => ir[i].op = IROp::AVX512Div,
                IROp::Load(_) => ir[i].op = IROp::AVX512Load,
                IROp::Store(_) => ir[i].op = IROp::AVX512Store,
                IROp::Mov => ir[i].op = IROp::AVX512Mov,
                _ => {}
            }
        }
    }
}

// Check if a function has array operations that can benefit from AVX512
fn has_array_operations(f: &Function) -> bool {
    for ir in &f.ir {
        if matches!(ir.op, IROp::Load(_) | IROp::Store(_)) {
            return true;
        }
    }
    false
}

// Detect math function calls that can be replaced with SIMD instructions
fn optimize_math_functions(ir: &mut [IR]) {
    let mut i = 0;
    while i < ir.len() {
        if let IROp::Call(ref name, nargs, _) = ir[i].op {
            let _lhs = ir[i].lhs.unwrap_or(0);
            
            // Replace common math functions with SIMD instructions
            match name.as_str() {
                "sqrt" if nargs == 1 => {
                    ir[i].op = IROp::AVX512Sqrt;
                },
                "fabs" if nargs == 1 => {
                    // Use bitwise operation to clear sign bit
                    ir[i].op = IROp::AVX512And;
                },
                "fmax" if nargs == 2 => {
                    ir[i].op = IROp::AVX512Max;
                },
                "fmin" if nargs == 2 => {
                    ir[i].op = IROp::AVX512Min;
                },
                // Add more math function optimizations
                _ => {}
            }
        }
        i += 1;
    }
}

// Main vectorization function
pub fn vectorize(fns: &mut Vec<Function>) {
    for f in fns {
        // Strategy 1: Check for vectorizable loops
        if can_vectorize_loop(&f.ir) {
            convert_to_avx512(&mut f.ir);
            continue;
        }
        
        // Strategy 2: Check for array operations
        if has_array_operations(f) {
            let vector_ops = identify_array_operations(&f.ir);
            if !vector_ops.is_empty() {
                vectorize_ranges(&mut f.ir, &vector_ops);
            }
        }
        
        // Strategy 3: Check for reduction patterns
        let reductions = detect_reduction_patterns(&f.ir);
        if !reductions.is_empty() {
            for (start, end, op) in reductions {
                // Convert the reduction pattern to use SIMD
                for i in start..=end {
                    if i == start + 1 {
                        // Clone the op to avoid the move error
                        f.ir[i].op = op.clone();
                    } else if i != start && i != end {
                        f.ir[i].op = IROp::Nop;
                    }
                }
            }
        }
        
        // Strategy 4: Optimize math function calls
        optimize_math_functions(&mut f.ir);
    }
} 