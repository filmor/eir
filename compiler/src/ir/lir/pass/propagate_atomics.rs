
use ::eir::{ Value, ConstantTerm };
use ::eir::op::OpKind;
use ::eir::FunctionBuilder;
use ::std::collections::{ HashMap, HashSet };

pub fn propagate_atomics(builder: &mut FunctionBuilder) {

    // Write => Read
    let mut constants: HashMap<SSAVariable, ConstantTerm> = HashMap::new();
    let mut moves: HashMap<SSAVariable, SSAVariable> = HashMap::new();
    let mut phi_reads: HashSet<SSAVariable> = HashSet::new();

    for block_container in cfg.graph.nodes() {
        let mut block = block_container.inner.borrow_mut();
        for phi in block.phi_nodes.iter() {
            for (_edge, ssa) in phi.entries.iter() {
                phi_reads.insert(*ssa);
            }
        }
    }

    for block_container in cfg.graph.nodes() {
        let mut block = block_container.inner.borrow_mut();
        block.ops.retain(|op| {
            match op.kind {
                // Moves can be removed if they are not referenced in any
                // PHI reads.
                OpKind::Move if !phi_reads.contains(&op.writes[0]) => {
                    match op.reads[0] {
                        Source::Constant(ref constant) => {
                            constants.insert(op.writes[0], constant.clone());
                            false
                        }
                        Source::Variable(ssa) => {
                            moves.insert(op.writes[0], ssa);
                            false
                        }
                    }
                }
                // UnpackValueList with 0 or 1 writes can be removed
                OpKind::UnpackValueList => {
                    if op.writes.len() == 0 {
                        false
                    } else if op.writes.len() == 1 {
                        match op.reads[0] {
                            Source::Constant(ref constant) => {
                                constants.insert(op.writes[0], constant.clone());
                                false
                            }
                            Source::Variable(ssa) => {
                                moves.insert(op.writes[0], ssa);
                                false
                            }
                        }
                    } else {
                        true
                    }
                }
                _ => true,
            }
        });
    }

    let mut moves_prop: HashMap<SSAVariable, SSAVariable> = moves.clone();
    for (o_write, _) in moves.iter() {
        loop {
            let o_read = *moves_prop.get(o_write).unwrap();
            if let Some(i_read) = moves_prop.get(&o_read).cloned() {
                moves_prop.insert(*o_write, i_read);
            } else {
                break;
            }
        }
    }

    for block_container in cfg.graph.nodes() {
        let mut block = block_container.inner.borrow_mut();
        //for phi in block.phi_nodes.iter_mut() {
        //    for (_edge, ref mut ssa) in phi.entries.iter_mut() {
        //        if let Source::Variable(var) = entry.1 {
        //            if let Some(constant) = constants.get(&var) {
        //                entry.1 = Source::Constant(constant.clone());
        //            } else {
        //                if let Some(rssa) = moves_prop.get(&var) {
        //                    if let Some(constant) = constants.get(rssa) {
        //                        entry.1 = Source::Constant(constant.clone());
        //                    } else {
        //                        entry.1 = Source::Variable(*rssa);
        //                    }
        //                }
        //            }
        //        }
        //    }
        //}
        for op in block.ops.iter_mut() {
            for read in op.reads.iter_mut() {
                if let Source::Variable(var) = *read {
                    if let Some(constant) = constants.get(&var) {
                        *read = Source::Constant(constant.clone());
                    } else {
                        if let Some(rssa) = moves_prop.get(&var) {
                            if let Some(constant) = constants.get(rssa) {
                                *read = Source::Constant(constant.clone());
                            } else {
                                *read = Source::Variable(*rssa);
                            }
                        }
                    }
                }
            }
        }
    }
}
