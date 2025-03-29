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

// Convert regular IR operations to AVX512 operations
fn convert_to_avx512(ir: &mut [IR]) {
    for i in 0..ir.len() {
        match ir[i].op {
            // Convert floating-point operations
            IROp::Add => ir[i].op = IROp::AVX512Add,
            IROp::Sub => ir[i].op = IROp::AVX512Sub,
            IROp::Mul => ir[i].op = IROp::AVX512Mul,
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
    }
} 