use std::cmp::Ordering;
use rand::Rng;


fn main() {
    let mut rng = rand::thread_rng();

    const ARRAY_LENGTH: usize = 10000;
    let mut generate_random_sequence_i32 = || {
        (0..ARRAY_LENGTH)
            .map(|_| rng.gen_range(0..i32::MAX))
            .collect::<Vec<_>>()
    };

    let mut tests =
        (0..10).map(|_| generate_random_sequence_i32()).collect::<Vec<_>>();

    let mut sorted_bubble = tests.clone();
    println!(
        "bubble sort took: {:?}",
        timed_sort(&mut sorted_bubble, bubble_sort)
    );

    let mut sorted_select = tests.clone();
    println!(
        "selection sort took: {:?}",
        timed_sort(&mut sorted_select, selection_sort)
    );

    let mut sorted_insertion = tests.clone();
    println!(
        "insertion sort took: {:?}",
        timed_sort(&mut sorted_insertion, insertion_sort)
    );

    let (sorted_merge, merge_sort_time) =
        timed_sort2(tests.clone(), merge_sort);
    println!("merge sort took: {:?}", merge_sort_time);

    let mut sorted_merge2 = tests.clone();
    println!(
        "merge sort2 took: {:?}",
        timed_sort(&mut sorted_merge2, merge_sort2)
    );

    let mut sorted_merge3 = tests.clone();
    println!(
        "merge sort3 took: {:?}",
        timed_sort(&mut sorted_merge3, merge_sort3)
    );

    let mut sorted_quick = tests.clone();
    println!(
        "quick sort took: {:?}",
        timed_sort(&mut sorted_quick, quick_sort)
    );

    fn rust_sort(v: &mut [impl Ord]) {
        v.sort();
    }
    println!(
        "inbuilt merge sort took: {:?}",
        timed_sort(&mut tests, rust_sort)
    );

    assert_eq!(sorted_bubble, tests);
    assert_eq!(sorted_select, tests);
    assert_eq!(sorted_insertion, tests);
    assert_eq!(sorted_merge, tests);
    assert_eq!(sorted_merge2, tests);
    assert_eq!(sorted_merge3, tests);
    assert_eq!(sorted_quick, tests);
}


fn bubble_sort<T: Ord>(slice: &mut [T]) {
    for i in 1..=slice.len() {
        for j in 0..(slice.len() - i) {
            match slice[j].cmp(&slice[j + 1]) {
                Ordering::Less | Ordering::Equal => {
                    // Do nothing
                }
                Ordering::Greater => {
                    slice.swap(j, j + 1);
                }
            }
        }
    }
}


fn selection_sort<T: Ord>(slice: &mut [T]) {
    for i in 0..slice.len() {
        let mut selected = i;
        for j in (i + 1)..slice.len() {
            if slice[j] < slice[selected] {
                selected = j;
            }
        }

        slice.swap(i, selected);
    }
}


fn insertion_sort<T: Ord>(slice: &mut [T]) {
    for i in 1..slice.len() {
        for j in (1..=i).rev() {
            if slice[j] < slice[j - 1] {
                slice.swap(j, j - 1);
            } else {
                break;
            }
        }
    }
}


/// 实现 1:
/// 此实现需要大量浅复制(move)，但不需要深拷贝，对primitive type排序较慢，但对没有 Copy Trait 的类型较快
fn merge_sort<T: Ord>(mut v: Vec<T>) -> Vec<T> {
    if v.len() < 2 {
        return v;
    }

    // Split the right half and sort them first
    let mut right = merge_sort(v.split_off(v.len() / 2));
    let mut left = merge_sort(v);

    let mut result = Vec::new();

    // 反向merge，因为 `Vec::remove(0)` 的复杂度是 `O(n)` 而且需要大量复制
    while !left.is_empty() && !right.is_empty() {
        if left.last().unwrap() > right.last().unwrap() {
            result.push(left.pop().unwrap());
        } else {
            result.push(right.pop().unwrap());
        }
    }
    result.extend(left.into_iter().rev());
    result.extend(right.into_iter().rev());
    result.reverse();

    result
}


/// 实现 2:
/// 此实现使用了相对更少的复制，但需要更多的深拷贝。Primitive type排序速度更快，需要深拷贝的类型速度更慢
fn merge_sort2<T: Ord + Clone>(v: &mut [T]) {
    if v.len() < 2 {
        return;
    }

    let mid_idx = v.len() / 2;

    merge_sort2(&mut v[..mid_idx]);
    merge_sort2(&mut v[mid_idx..]);

    let mut temporary = Vec::with_capacity(v.len());

    let mut l = 0;
    let mut r = mid_idx;

    while l < mid_idx && r < v.len() {
        if v[l] < v[r] {
            temporary.push(v[l].clone());
            l += 1;
        } else {
            temporary.push(v[r].clone());
            r += 1;
        }
    }
    temporary.extend(v[l..mid_idx].iter().cloned());
    temporary.extend(v[r..].iter().cloned());

    for (item, dest) in temporary.into_iter().zip(v.iter_mut()) {
        *dest = item;
    }
}


/// 实现 3:
/// 此实现类似实现2，但使用 unsafe 避免了深拷贝，性能优于实现1与实现2
fn merge_sort3<T: Ord>(v: &mut [T]) {
    if v.len() < 2 {
        return;
    }

    // 可以在遇到较短的数组时使用插入排序，性能较佳。但即使不使用插入排序，此实现性能依然优于实现1与实现2
    if v.len() < 32 {
        insertion_sort(v);
        return;
    }

    let mid_idx = v.len() / 2;

    merge_sort3(&mut v[..mid_idx]);
    merge_sort3(&mut v[mid_idx..]);

    let alloc_array = |size: usize| -> *mut T {
        // 等同于C中: `(T*)malloc(sizeof(T) * size)`
        unsafe {
            std::alloc::alloc(
                std::alloc::Layout::array::<T>(size).unwrap_unchecked(),
            ) as *mut T
        }
    };
    let dealloc_array = |ptr: *mut T, size: usize| unsafe {
        // 等同于C中: `free(ptr)`
        std::alloc::dealloc(
            ptr as *mut u8,
            std::alloc::Layout::array::<T>(size).unwrap_unchecked(),
        )
    };

    let temporary = alloc_array(v.len());
    let mut used_len = 0;

    let mut l = 0;
    let mut r = mid_idx;

    unsafe {
        while l < mid_idx && r < v.len() {
            if v[l] < v[r] {
                temporary
                    .add(used_len)
                    .copy_from_nonoverlapping(v.as_ptr().add(l), 1);
                l += 1;
            } else {
                temporary
                    .add(used_len)
                    .copy_from_nonoverlapping(v.as_ptr().add(r), 1);
                r += 1;
            }
            used_len += 1;
        }

        let left_remain = mid_idx - l;
        temporary
            .add(used_len)
            .copy_from_nonoverlapping(v.as_ptr().add(l), left_remain);
        used_len += left_remain;

        let right_remain = v.len() - r;
        temporary
            .add(used_len)
            .copy_from_nonoverlapping(v.as_ptr().add(r), right_remain);

        v.as_mut_ptr().copy_from_nonoverlapping(temporary, v.len());
    }

    dealloc_array(temporary, v.len());
}



fn quick_sort<T: Ord>(slice: &mut [T]) {
    const PIVOT: usize = 0;

    match slice.len().cmp(&2) {
        Ordering::Less => {}
        Ordering::Equal => {
            if slice[0] > slice[1] {
                slice.swap(0, 1);
            }
        }
        Ordering::Greater => {
            let mut swap_point = 0;
            for i in 1..slice.len() {
                if slice[i] < slice[PIVOT] {
                    swap_point += 1;
                    slice.swap(swap_point, i);
                }
            }
            slice.swap(PIVOT, swap_point);

            quick_sort(&mut slice[..swap_point]);
            quick_sort(&mut slice[(swap_point + 1)..]);
        }
    }
}


fn timed_sort<T, F>(values: &mut [Vec<T>], sort_fn: F) -> std::time::Duration
    where
        T: Ord,
        F: Fn(&mut [T]),
    {
        let start = std::time::Instant::now();
        for v in values.iter_mut() {
            sort_fn(v.as_mut_slice());
        }

        start.elapsed() / values.len() as u32
    }

    fn timed_sort2<T, F>(
        values: Vec<Vec<T>>,
        sort_fn: F,
    ) -> (Vec<Vec<T>>, std::time::Duration)
    where
        T: Ord,
        F: Fn(Vec<T>) -> Vec<T>,
    {
        let v_len = values.len() as u32;

        let start = std::time::Instant::now();
        let result = values.into_iter().map(sort_fn).collect();
        (result, start.elapsed() / v_len)
    }
