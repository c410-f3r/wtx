macro_rules! as_ptr_doc {
  ($struct:literal, $new:literal) => {
    concat!(
      "Returns a raw pointer to the slice’s buffer.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::",
      $struct,
      "::from_iterator(",
      $new,
      ").unwrap();\n",
      "let ptr = instance.as_ptr();\n",
      "for idx in 0..instance.len() {\n",
      "  let lhs = instance.get(idx).unwrap();\n",
      "  let rhs = unsafe { &*ptr.add(idx) };\n",
      "  assert_eq!(lhs, rhs);\n",
      "}\n",
      "```"
    )
  };
}

macro_rules! as_ptr_mut_doc {
  () => {
    "Returns a raw mutable pointer to the slice's buffer."
  };
}

macro_rules! as_slice_doc {
  ($struct:literal, $new:literal, $slice:literal) => {
    concat!(
      "Extracts a slice containing the entire instance.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::",
      $struct,
      "::from_iterator(",
      $new,
      ").unwrap();\n",
      "assert_eq!(instance.as_slice(), ",
      $slice,
      ");\n",
      "```"
    )
  };
}

macro_rules! as_slice_mut_doc {
  () => {
    concat!("Extracts a mutable slice containing the entire instance.\n",)
  };
}

macro_rules! capacity_doc {
  ($struct:literal, $new:literal) => {
    concat!(
      "Returns the total number of elements the instance can hold without reallocating.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::",
      $struct,
      "::from_iterator(",
      $new,
      ").unwrap();\n",
      "assert!(instance.capacity() >= 3);\n",
      "```"
    )
  };
}

macro_rules! clear_doc {
  ($struct:literal, $new:literal) => {
    concat!(
      "Clears the instance, removing all values.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::",
      $struct,
      "::from_iterator(",
      $new,
      ").unwrap();\n",
      "instance.clear();\n",
      "assert_eq!(instance.len(), 0);\n",
      "```"
    )
  };
}

macro_rules! expand_doc {
  ($struct:literal) => {
    concat!(
      "Resizes the instance in-place so that the current length is equal to `et`.\n",
      "Does nothing if the calculated length is equal or less than the current length.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::",
      $struct,
      "::new();\n",
      "instance.expand(wtx::collection::ExpansionTy::Len(3), 0).unwrap();\n",
      "assert_eq!(instance.as_slice(), &[0, 0, 0]);\n",
      "```"
    )
  };
}

macro_rules! extend_from_cloneable_slice_doc {
  ($struct:literal) => {
    concat!(
      "Iterates over the slice `other`, clones each element and then appends it to this instance. The `other` slice is traversed in-order.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::", $struct ,"::new();\n",
      "instance.extend_from_cloneable_slice(&[1, 2, 3]).unwrap();\n",
      "assert_eq!(instance.as_slice(), &[1, 2, 3]);\n",
      "```"
    )
  };
}

macro_rules! extend_from_copyable_slice_doc {
  ($struct:literal) => {
    concat!(
      "Iterates over the slice `other`, copies each element and then appends it to this instance. The `other` slice is traversed in-order.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::", $struct ,"::new();\n",
      "instance.extend_from_copyable_slice(&[1, 2, 3]).unwrap();\n",
      "assert_eq!(instance.as_slice(), &[1, 2, 3]);\n",
      "```"
    )
  };
}

macro_rules! extend_from_iter_doc {
  ($struct:literal, $new:literal, $slice:literal) => {
    concat!(
      "Appends all elements of the iterator.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::",
      $struct,
      "::new();\n",
      "instance.extend_from_iter(",
      $new,
      ").unwrap();\n",
      "assert_eq!(instance.as_slice(), ",
      $slice,
      ");\n",
      "```"
    )
  };
}

macro_rules! from_cloneable_elem_doc {
  ($struct:literal) => {
    concat!(
      "Constructs a new instance with elements provided by `iter`.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::",
      $struct,
      "::from_cloneable_elem(3, 0).unwrap();\n",
      "assert_eq!(instance.as_slice(), &[0, 0, 0]);\n",
      "```"
    )
  };
}

macro_rules! from_cloneable_slice_doc {
  ($struct:literal) => {
    concat!(
      "Creates a new instance from the cloneable elements of `slice`.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::",
      $struct,
      "::from_cloneable_slice(&[1, 2, 3]).unwrap();\n",
      "assert_eq!(instance.as_slice(), &[1, 2, 3]);\n",
      "```"
    )
  };
}

macro_rules! from_copyable_slice_doc {
  ($struct:literal) => {
    concat!(
      "Creates a new instance from the copyable elements of `slice`.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::",
      $struct,
      "::from_copyable_slice(&[1, 2, 3]).unwrap();\n",
      "assert_eq!(instance.as_slice(), &[1, 2, 3]);\n",
      "```"
    )
  };
}

macro_rules! from_iter_doc {
  ($struct:literal, $new:literal, $slice:literal) => {
    concat!(
      "Constructs a new instance with elements provided by `iter`.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::",
      $struct,
      "::from_iterator(",
      $new,
      ").unwrap();\n",
      "assert_eq!(instance.as_slice(), ",
      $slice,
      ");\n",
      "```"
    )
  };
}

macro_rules! len_doc {
  () => {
    "Returns the number of elements in the storage, also referred to as its ‘length’."
  };
}

macro_rules! pop_doc {
  ($struct:literal, $new:literal, $slice:literal) => {
    concat!(
      "Removes the last element from a vector and returns it, or `None` if it is empty.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::",
      $struct,
      "::from_iterator(",
      $new,
      ").unwrap();\n",
      "assert!(instance.pop().is_some());\n",
      "assert_eq!(instance.as_slice(), ",
      $slice,
      ");\n",
      "```"
    )
  };
}

macro_rules! push_doc {
  ($struct:literal, $elem:literal, $slice:literal) => {
    concat!(
      "Appends an element to the back of the collection.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::",
      $struct,
      "::new();\n",
      "instance.push(",
      $elem,
      ").unwrap();\n",
      "assert_eq!(instance.as_slice(), ",
      $slice,
      ");\n",
      "```"
    )
  };
}

macro_rules! remaining_doc {
  ($struct:literal, $elem:literal) => {
    concat!(
      "How many elements can be added to this collection.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::",
      $struct,
      "::new();\n",
      "instance.push(",
      $elem,
      ").unwrap();\n",
      "let remaining = instance.capacity().wrapping_sub(instance.len());\n",
      "assert_eq!(instance.remaining(), remaining);\n",
      "```"
    )
  };
}

macro_rules! remove_doc {
  ($struct:literal, $new:literal, $slice:literal) => {
    concat!(
      "Removes and returns the element at position index within the instance, shifting all elements after it to the left.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::", $struct, "::from_iterator(", $new, ").unwrap();\n",
      "instance.remove(1).unwrap();\n",
      "assert_eq!(instance.as_slice(), ", $slice, ");\n",
      "```"
    )
  };
}

macro_rules! reserve_doc {
  ($struct:literal) => {
    concat!(
      "Reserves capacity for at least additional more elements to be inserted in the given instance.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::", $struct, "::new();\n",
      "instance.reserve(10).unwrap();\n",
      "assert!(instance.capacity() >= 10);\n",
      "```"
    )
  };
}

macro_rules! reserve_exact_doc {
  ($struct:literal) => {
    concat!(
      "Tries to reserve the minimum capacity for at least `additional`\n",
      "elements to be inserted in the given instance. Unlike [`Self::reserve`],\n",
      "this will not deliberately over-allocate to speculatively avoid frequent\n",
      "allocations.",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::",
      $struct,
      "::new();\n",
      "instance.reserve_exact(10).unwrap();\n",
      "assert!(instance.capacity() >= 10);\n",
      "```"
    )
  };
}

macro_rules! set_len_doc {
  () => {
    concat!(
      "Forces the length of the instance to `new_len`.\n\n",
      "# Safety\n\n",
      "- `new_len` must be less than or equal to the current capacity.\n",
      "- Elements up to `new_len` must be initialized.",
    )
  };
}

macro_rules! truncate_doc {
  ($struct:literal, $new:literal, $slice:literal) => {
    concat!(
      "Shortens the instance, keeping the first len elements and dropping the rest.\n",
      "\n",
      "```rust\n",
      "let mut instance = wtx::collection::",
      $struct,
      "::from_iterator(",
      $new,
      ").unwrap();\n",
      "instance.truncate(1);\n",
      "assert_eq!(instance.as_slice(), ",
      $slice,
      ");\n",
      "```"
    )
  };
}
