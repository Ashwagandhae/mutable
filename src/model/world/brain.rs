use std::{collections::HashMap, fmt::Display};

use super::{
    collection::{CollectionView, GenId},
    gene::{BuildGene, BuildId, Gene, Mutation},
    node::{Node, NodeKind, ENERGY_LOSS_RATE},
};
use int_enum::IntEnum;
use rand::random;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum ConnectSource {
    Neuron(NeuronsIndex),
    Bias,
}

#[derive(Debug, Clone)]
pub struct Connect {
    from: ConnectSource,
    to: NeuronsIndex,
    weight: f32,
    enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
struct NeuronsIndex {
    index: usize,
    kind: NeuronsIndexKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NeuronsIndexKind {
    Input,
    Synth,
    Output,
    Hidden,
}

#[derive(Debug, Clone)]
pub struct Neurons {
    inputs: Vec<NeuronKind>,
    synths: Vec<NeuronKind>,
    outputs: Vec<NeuronKind>,
    hiddens: Vec<NeuronKind>,
}

impl Neurons {
    fn new() -> Self {
        Self {
            inputs: Vec::new(),
            synths: Vec::new(),
            outputs: Vec::new(),
            hiddens: Vec::new(),
        }
    }
    fn iter(&self) -> impl Iterator<Item = &NeuronKind> {
        self.inputs
            .iter()
            .chain(self.synths.iter())
            .chain(self.outputs.iter())
            .chain(self.hiddens.iter())
    }
    fn iter_index(&self) -> impl Iterator<Item = (NeuronsIndex, &NeuronKind)> {
        use NeuronsIndexKind::*;
        self.inputs
            .iter()
            .enumerate()
            .map(|(i, n)| {
                (
                    NeuronsIndex {
                        index: i,
                        kind: Input,
                    },
                    n,
                )
            })
            .chain(self.synths.iter().enumerate().map(|(i, n)| {
                (
                    NeuronsIndex {
                        index: i,
                        kind: Synth,
                    },
                    n,
                )
            }))
            .chain(self.outputs.iter().enumerate().map(|(i, n)| {
                (
                    NeuronsIndex {
                        index: i,
                        kind: Output,
                    },
                    n,
                )
            }))
            .chain(self.hiddens.iter().enumerate().map(|(i, n)| {
                (
                    NeuronsIndex {
                        index: i,
                        kind: Hidden,
                    },
                    n,
                )
            }))
    }

    fn add_input(&mut self, id: BuildId) -> NeuronsIndex {
        self.inputs.push(NeuronKind::Input(id));
        NeuronsIndex {
            index: self.inputs.len() - 1,
            kind: NeuronsIndexKind::Input,
        }
    }
    fn add_synth(&mut self, amp: f32, freq: f32) -> NeuronsIndex {
        self.synths.push(NeuronKind::Synth { amp, freq });
        NeuronsIndex {
            index: self.synths.len() - 1,
            kind: NeuronsIndexKind::Synth,
        }
    }
    fn add_output(&mut self, id: BuildId) -> NeuronsIndex {
        self.outputs.push(NeuronKind::Output(id));
        NeuronsIndex {
            index: self.outputs.len() - 1,
            kind: NeuronsIndexKind::Output,
        }
    }
    fn add_hidden(&mut self) -> NeuronsIndex {
        self.hiddens.push(NeuronKind::Hidden);
        NeuronsIndex {
            index: self.hiddens.len() - 1,
            kind: NeuronsIndexKind::Hidden,
        }
    }
    fn vec_from_kind(&self, kind: &NeuronsIndexKind) -> &Vec<NeuronKind> {
        use NeuronsIndexKind::*;
        match kind {
            Input => &self.inputs,
            Synth => &self.synths,
            Output => &self.outputs,
            Hidden => &self.hiddens,
        }
    }
    fn vec_from_kind_mut(&mut self, kind: &NeuronsIndexKind) -> &mut Vec<NeuronKind> {
        use NeuronsIndexKind::*;
        match kind {
            Input => &mut self.inputs,
            Synth => &mut self.synths,
            Output => &mut self.outputs,
            Hidden => &mut self.hiddens,
        }
    }
    fn random_index(&self, kinds: &[NeuronsIndexKind]) -> Option<NeuronsIndex> {
        let range: usize = kinds
            .iter()
            .map(|kind| self.vec_from_kind(kind).len())
            .sum();
        if range == 0 {
            return None;
        }
        let mut index = random::<usize>() % range;
        for kind in kinds {
            let vec = self.vec_from_kind(kind);
            if index < vec.len() {
                return Some(NeuronsIndex {
                    index,
                    kind: kind.clone(),
                });
            }
            index -= vec.len();
        }
        unreachable!() // can't happen if range is > 0
    }
    // return index that was swapped due to removal, it is now at the index of the removed
    fn swap_remove(&mut self, index: NeuronsIndex) -> NeuronsIndex {
        let vec = self.vec_from_kind_mut(&index.kind);
        vec.swap_remove(index.index);
        NeuronsIndex {
            index: vec.len(),
            kind: index.kind,
        }
    }
    fn len(&self) -> usize {
        self.inputs.len() + self.synths.len() + self.outputs.len() + self.hiddens.len()
    }
    fn cost_len(&self) -> usize {
        self.hiddens.len() + self.synths.len()
    }
    fn index_to_usize(&self, index: NeuronsIndex) -> usize {
        use NeuronsIndexKind::*;
        match index.kind {
            Input => index.index,
            Synth => self.inputs.len() + index.index,
            Output => self.inputs.len() + self.synths.len() + index.index,
            Hidden => self.inputs.len() + self.synths.len() + self.outputs.len() + index.index,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BrainPlan {
    neurons: Neurons,
    connects: Vec<Connect>,
}

#[derive(Debug, Clone)]
pub struct Neuron {
    value: f32,
    prev_value: f32,
}

fn activate_hidden(x: f32) -> f32 {
    // use tanh
    x.tanh()
}

fn activate_output(x: f32) -> f32 {
    // use linear 💀
    x
}

#[derive(Debug, Clone)]
pub enum NeuronKind {
    Input(BuildId),
    Synth { amp: f32, freq: f32 },
    Hidden,
    Output(BuildId),
}

impl BrainPlan {
    pub fn new() -> Self {
        let neurons = Neurons::new();
        let connects = Vec::new();

        Self { neurons, connects }
    }

    pub fn mutate_gene(&mut self, mutation: Mutation, gene: &Gene) {
        if let Gene::Build((gene, id)) = gene {
            match mutation {
                Mutation::Add => self.add_build_gene(*id, gene),
                Mutation::Delete => self.delete_build_gene(*id),
                Mutation::Edit | Mutation::EditGradual => self.edit_build_gene(*id, gene),
            }
        }
    }

    fn gene_input_worthy(gene: &BuildGene) -> bool {
        gene.has_sense == 1 || NodeKind::from_int(gene.node_kind).unwrap().is_input()
    }

    fn gene_output_worthy(gene: &BuildGene) -> bool {
        gene.has_muscle == 1 || NodeKind::from_int(gene.node_kind).unwrap().is_output()
    }

    fn add_build_gene(&mut self, id: BuildId, gene: &BuildGene) {
        if Self::gene_input_worthy(gene) {
            self.add_input(id);
        }
        if Self::gene_output_worthy(gene) {
            self.add_output(id);
        }
    }

    fn delete_build_gene(&mut self, id: BuildId) {
        self.delete_input(id);
        self.delete_output(id);
    }

    fn edit_build_gene(&mut self, id: BuildId, gene: &BuildGene) {
        let old_input = self.neurons.iter_index().find(|(_, n)| match n {
            NeuronKind::Input(neuron_id) => *neuron_id == id,
            _ => false,
        });
        match old_input {
            Some((index, _)) if !Self::gene_input_worthy(gene) => self.delete_neuron(index),
            None if Self::gene_input_worthy(gene) => self.add_input(id),
            _ => {}
        }
        let old_output = self.neurons.iter_index().find(|(_, n)| match n {
            NeuronKind::Output(neuron_id) => *neuron_id == id,
            _ => false,
        });
        match old_output {
            Some((index, _)) if !Self::gene_output_worthy(gene) => self.delete_neuron(index),
            None if Self::gene_output_worthy(gene) => self.add_output(id),
            _ => {}
        }
    }

    fn add_input(&mut self, id: BuildId) {
        let index = self.neurons.add_input(id);
        if random::<f32>() > 0.75 {
            self.add_random_connect(Some(ConnectSource::Neuron(index)), None);
        }
    }

    fn add_output(&mut self, id: BuildId) {
        let index = self.neurons.add_output(id);
        if random::<f32>() > 0.75 {
            self.add_random_connect(None, Some(index));
        }
    }

    fn delete_input(&mut self, id: BuildId) {
        loop {
            let Some(delete_neuron_index) = self
                .neurons
                .iter_index()
                .find(|(_, n)| match n {
                    NeuronKind::Input(neuron_id) => *neuron_id == id,
                    _ => false,
                })
                .map(|(index, _)| index) else { break };
            self.delete_neuron(delete_neuron_index);
        }
    }

    fn delete_output(&mut self, id: BuildId) {
        loop {
            let Some(delete_neuron_index) = self
                .neurons
                .iter_index()
                .find(|(_, n)| match n {
                    NeuronKind::Output(neuron_id) => *neuron_id == id,
                    _ => false,
                })
                .map(|(index, _)| index) else { break };
            self.delete_neuron(delete_neuron_index);
        }
    }

    fn mutate_add_input_wave(&mut self) {
        let amp = random::<f32>() * 2.0 - 1.0;
        let freq = random::<f32>() * 0.1 + 0.01;
        self.neurons.add_synth(amp, freq);
    }

    fn add_random_connect(&mut self, from: Option<ConnectSource>, to: Option<NeuronsIndex>) {
        use NeuronsIndexKind::*;
        let from = match from {
            Some(source) => source,
            None => {
                if random::<f32>() > 0.4 {
                    let Some(index) = self.neurons.random_index(&[Input, Synth, Hidden]) else { return };
                    ConnectSource::Neuron(index)
                } else {
                    ConnectSource::Bias
                }
            }
        };
        let to = match to {
            Some(index) => index,
            None => match self.neurons.random_index(&[Hidden, Output]) {
                Some(index) => index,
                None => return,
            },
        };
        let weight = random::<f32>() * 4.0 - 2.0;
        let enabled = true;
        self.connects.push(Connect {
            from,
            to,
            weight,
            enabled,
        });
    }
    fn mutate_add_connect(&mut self) {
        self.add_random_connect(None, None);
    }
    fn mutate_add_neuron(&mut self) {
        if self.connects.is_empty() {
            return;
        }
        let connect = self.connects[random::<usize>() % self.connects.len()].clone();

        let index = self.neurons.add_hidden();
        let new_connect_1 = Connect {
            from: connect.from,
            to: index,
            weight: 1.0,
            enabled: true,
        };
        let new_connect_2 = Connect {
            from: ConnectSource::Neuron(index),
            to: connect.to,
            weight: connect.weight,
            enabled: connect.enabled,
        };

        self.connects.push(new_connect_1);
        self.connects.push(new_connect_2);
    }
    fn mutate_delete_connect(&mut self) {
        if self.connects.is_empty() {
            return;
        }
        self.connects
            .swap_remove(random::<usize>() % self.connects.len());
    }
    fn delete_neuron(&mut self, index: NeuronsIndex) {
        // remove all connects to and from this neuron
        self.connects
            .retain(|c| c.from != ConnectSource::Neuron(index) && c.to != index);

        let swapped_index = self.neurons.swap_remove(index);

        // fix index of swap removed
        for connect in &mut self.connects {
            if let ConnectSource::Neuron(ref mut from) = connect.from {
                if *from == swapped_index {
                    *from = index;
                }
            }
            if connect.to == swapped_index {
                connect.to = index;
            }
        }
    }
    fn mutate_delete_neuron(&mut self) {
        self.neurons
            .random_index(&[NeuronsIndexKind::Synth, NeuronsIndexKind::Hidden])
            .map(|index| self.delete_neuron(index));
    }
    fn mutate_enable_disable(&mut self) {
        if self.connects.is_empty() {
            return;
        }
        let len = self.connects.len();
        let connect = &mut self.connects[random::<usize>() % len];
        connect.enabled = !connect.enabled;
    }
    fn mutate_weight_shift(&mut self) {
        if self.connects.is_empty() {
            return;
        }
        let len = self.connects.len();
        let connect = &mut self.connects[random::<usize>() % len];
        connect.weight += random::<f32>() * 2.0 - 1.0;
    }
    fn mutate_weight_random(&mut self) {
        if self.connects.is_empty() {
            return;
        }
        let len = self.connects.len();
        let connect = &mut self.connects[random::<usize>() % len];
        connect.weight = random::<f32>() * 2.0 - 1.0;
    }
    pub fn mutate(&mut self) {
        let mutations = [
            Self::mutate_add_input_wave,
            Self::mutate_add_connect,
            Self::mutate_add_neuron,
            Self::mutate_delete_connect,
            Self::mutate_delete_neuron,
            Self::mutate_enable_disable,
            Self::mutate_weight_shift,
            Self::mutate_weight_random,
        ];
        let mutation = mutations[random::<usize>() % mutations.len()];
        mutation(self);
    }
    pub fn cost(&self) -> f32 {
        self.neurons.cost_len() as f32 * ENERGY_LOSS_RATE * 0.015625
    }
}

impl Display for BrainPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, neuron) in self.neurons.iter().enumerate() {
            writeln!(f, "{}: {:?}", i, neuron)?;
        }
        for (i, connect) in self.connects.iter().enumerate() {
            writeln!(f, "{}: {:?}", i, connect)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum NeuronCalculate {
    Skip,
    Calculate(Vec<Neuron>),
}

#[derive(Debug, Clone)]
pub struct Brain {
    neurons: NeuronCalculate,
}

impl Brain {
    pub fn from_plan(plan: &BrainPlan) -> Self {
        let connections_to_outputs = plan
            .connects
            .iter()
            .filter(|c| c.enabled)
            .filter(|c| matches!(c.to.kind, NeuronsIndexKind::Output))
            .count();
        Self {
            neurons: if connections_to_outputs == 0 {
                NeuronCalculate::Skip
            } else {
                NeuronCalculate::Calculate(vec![
                    Neuron {
                        value: 0.0,
                        prev_value: 0.0,
                    };
                    plan.neurons.len()
                ])
            },
        }
    }
    pub fn step(
        &mut self,
        plan: &BrainPlan,
        build_id_map: &HashMap<BuildId, GenId>,
        nodes: &mut CollectionView<Node>,
        tick: u64,
    ) -> f32 {
        let cost = plan.cost();
        let neurons = match &mut self.neurons {
            NeuronCalculate::Skip => return cost,
            NeuronCalculate::Calculate(neurons) => neurons,
        };
        // reset all neurons, and set input neurons to their input values
        let get_node_sense = |id: BuildId| {
            build_id_map
                .get(&id)
                // safe because no 2 organisms share nodes
                .and_then(|node_id| nodes.get(*node_id).and_then(|node| node.sense().map(|s| s)))
                .unwrap_or(0.0)
        };
        for (Neuron { value, prev_value }, kind) in neurons.iter_mut().zip(plan.neurons.iter()) {
            *prev_value = *value;
            *value = match kind {
                NeuronKind::Input(id) => get_node_sense(id.clone()),
                NeuronKind::Synth { amp, freq } => {
                    *amp * (2.0 * std::f32::consts::PI * tick as f32 * *freq).sin()
                }
                _ => 0.0,
            }
        }
        for Connect {
            from, to, weight, ..
        } in plan.connects.iter().filter(|c| c.enabled)
        {
            let from = match from {
                ConnectSource::Neuron(index) => {
                    neurons[plan.neurons.index_to_usize(*index)].prev_value
                }
                ConnectSource::Bias => 1.,
            };
            let to = &mut neurons[plan.neurons.index_to_usize(*to)].value;
            *to += from * weight;
        }

        for (Neuron { value, .. }, kind) in neurons.iter_mut().zip(plan.neurons.iter()) {
            let new_value = match kind {
                NeuronKind::Input(..) | NeuronKind::Synth { .. } => *value, // dont set activate for input neurons
                NeuronKind::Hidden => activate_hidden(*value),
                NeuronKind::Output(..) => activate_output(*value),
            };
            *value = new_value;

            if let NeuronKind::Output(id) = kind {
                let activate = *value;
                // safe because no 2 organisms share nodes
                build_id_map.get(&id).and_then(|node_id| {
                    nodes.get_mut(*node_id).map(|node| node.activate(activate))
                });
            }
        }
        // neurons use 1/8 of a node's energy
        cost
    }

    pub fn does_calculate_neurons(&self) -> bool {
        match &self.neurons {
            NeuronCalculate::Skip => false,
            NeuronCalculate::Calculate(_) => true,
        }
    }
}

impl Display for Brain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.neurons {
            NeuronCalculate::Skip => write!(f, "[no outputs]"),
            NeuronCalculate::Calculate(neurons) => {
                write!(f, "[")?;
                for neuron in neurons {
                    write!(f, "{:.2}, ", neuron.value)?;
                }
                write!(f, "]")
            }
        }
    }
}
